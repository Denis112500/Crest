//! Notices interrupted turns, denied permissions (both write the same line), stopped background
//! subagents and the session's title: while a session is busy (working, on its turn or its
//! subagents, or waiting for an OK), its transcript's folder is watched and only the newly added
//! lines are checked. Nothing is watched while no session is busy.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

use crate::activity_sources::claude_code::claude_code_session_title::{find_transcript_session_title, ClaudeCodeSessionTitle};
use crate::activity_sources::claude_code::claude_code_status_message::ClaudeCodeStatusMessage;
use crate::activity_sources::claude_code::claude_code_transcript_interrupt::{
    find_transcript_interrupt, ClaudeCodeTranscriptInterrupt,
};
use crate::activity_sources::claude_code::transcript_tail_reader::TranscriptTailReader;
use crate::folder_change_watching::{CurrentPlatformFolderChangeWatch, FolderChangeWatch};

/// What a new transcript line told about its session.
pub enum ClaudeCodeTranscriptNews {
    Interrupt(ClaudeCodeTranscriptInterrupt),
    SessionTitle(ClaudeCodeSessionTitle),
}

pub struct ClaudeCodeTranscriptWatch {
    transcript_tails_by_session_id: HashMap<String, TranscriptTailReader>,
    folder_watches_by_folder: HashMap<PathBuf, CurrentPlatformFolderChangeWatch>,
    status_message_sender: Sender<ClaudeCodeStatusMessage>,
}

impl ClaudeCodeTranscriptWatch {
    pub fn new(status_message_sender: Sender<ClaudeCodeStatusMessage>) -> Self {
        Self { transcript_tails_by_session_id: HashMap::new(), folder_watches_by_folder: HashMap::new(), status_message_sender }
    }

    /// Follows exactly the given busy sessions: new ones are read from their transcript's current
    /// end, sessions that are no longer busy are dropped, and so are unneeded folder watches.
    pub fn follow_busy_sessions(&mut self, busy_session_transcripts: Vec<(String, PathBuf)>) {
        self.transcript_tails_by_session_id.retain(|session_id, _| {
            busy_session_transcripts.iter().any(|(busy_session_id, _)| busy_session_id == session_id)
        });
        for (session_id, transcript_path) in busy_session_transcripts {
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
                // Not fatal: that session then only ends by a later event or the silence timeout,
                // and shows no title.
                Err(watch_error) => eprintln!("Crest: can't follow Claude Code's transcripts there: {watch_error}"),
            }
        }
    }

    /// The interrupts and titles in the new transcript lines of the followed sessions in
    /// `changed_folder`, in the order they were written.
    pub fn find_session_news(&mut self, changed_folder: &Path) -> Vec<(String, ClaudeCodeTranscriptNews)> {
        let mut session_news = Vec::new();
        for (session_id, tail_reader) in &mut self.transcript_tails_by_session_id {
            if tail_reader.transcript_path().parent() != Some(changed_folder) {
                continue;
            }
            // Every followed transcript in the folder is read, so each stays at its own end.
            for transcript_line in tail_reader.read_new_lines() {
                let transcript_news = find_transcript_interrupt(&transcript_line)
                    .map(ClaudeCodeTranscriptNews::Interrupt)
                    .or_else(|| find_transcript_session_title(&transcript_line).map(ClaudeCodeTranscriptNews::SessionTitle));
                if let Some(transcript_news) = transcript_news {
                    session_news.push((session_id.clone(), transcript_news));
                }
            }
        }
        session_news
    }
}
