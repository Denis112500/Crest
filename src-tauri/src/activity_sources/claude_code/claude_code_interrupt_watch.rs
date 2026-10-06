//! Notices interrupted turns: while a session is working, its transcript's folder is watched and
//! only the newly added lines are checked for the interrupt line. Nothing is watched while no
//! session works.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

use crate::activity_sources::claude_code::claude_code_status_message::ClaudeCodeStatusMessage;
use crate::activity_sources::claude_code::claude_code_transcript_interrupt::is_interrupt_transcript_line;
use crate::activity_sources::claude_code::transcript_tail_reader::TranscriptTailReader;
use crate::folder_change_watching::{CurrentPlatformFolderChangeWatch, FolderChangeWatch};

pub struct ClaudeCodeInterruptWatch {
    transcript_tails_by_session_id: HashMap<String, TranscriptTailReader>,
    folder_watches_by_folder: HashMap<PathBuf, CurrentPlatformFolderChangeWatch>,
    status_message_sender: Sender<ClaudeCodeStatusMessage>,
}

impl ClaudeCodeInterruptWatch {
    pub fn new(status_message_sender: Sender<ClaudeCodeStatusMessage>) -> Self {
        Self { transcript_tails_by_session_id: HashMap::new(), folder_watches_by_folder: HashMap::new(), status_message_sender }
    }

    /// Follows exactly the given working sessions: new ones are read from their transcript's
    /// current end, sessions that stopped working are dropped, and so are unneeded folder watches.
    pub fn follow_working_sessions(&mut self, working_session_transcripts: Vec<(String, PathBuf)>) {
        self.transcript_tails_by_session_id.retain(|session_id, _| {
            working_session_transcripts.iter().any(|(working_session_id, _)| working_session_id == session_id)
        });
        for (session_id, transcript_path) in working_session_transcripts {
            self.transcript_tails_by_session_id
                .entry(session_id)
                .or_insert_with(|| TranscriptTailReader::starting_at_end(&transcript_path));
        }
        let needed_folders: Vec<PathBuf> = self
            .transcript_tails_by_session_id
            .values()
            .filter_map(|tail_reader| tail_reader.transcript_path().parent().map(Path::to_path_buf))
            .collect();
        self.folder_watches_by_folder.retain(|watched_folder, _| needed_folders.contains(watched_folder));
        for needed_folder in needed_folders {
            if self.folder_watches_by_folder.contains_key(&needed_folder) {
                continue;
            }
            let status_message_sender = self.status_message_sender.clone();
            let changed_folder = needed_folder.clone();
            let on_folder_change = Box::new(move || {
                let _ = status_message_sender.send(ClaudeCodeStatusMessage::TranscriptFolderChanged(changed_folder.clone()));
            });
            match CurrentPlatformFolderChangeWatch::start_watching(&needed_folder, on_folder_change) {
                Ok(folder_watch) => {
                    self.folder_watches_by_folder.insert(needed_folder, folder_watch);
                }
                // Not fatal: that session then only ends by a later event or the silence timeout.
                Err(watch_error) => eprintln!("Crest: can't notice Claude Code interrupts there: {watch_error}"),
            }
        }
    }

    /// The followed sessions in `changed_folder` whose new transcript lines include an interrupt.
    pub fn find_interrupted_sessions(&mut self, changed_folder: &Path) -> Vec<String> {
        let mut interrupted_session_ids = Vec::new();
        for (session_id, tail_reader) in &mut self.transcript_tails_by_session_id {
            if tail_reader.transcript_path().parent() != Some(changed_folder) {
                continue;
            }
            // Every followed transcript in the folder is read, so each stays at its own end.
            if tail_reader.read_new_lines().iter().any(|line| is_interrupt_transcript_line(line)) {
                interrupted_session_ids.push(session_id.clone());
            }
        }
        interrupted_session_ids
    }
}
