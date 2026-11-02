//! "Answer permission requests in the pill", shared by the settings window and the running
//! Claude Code source. Off, Crest adds no `PermissionRequest` hook; a session started before the
//! change may still send one, and it gets "no decision" at once, so the pill never offers Allow.
//! It also tells the settings window how many requests wait in the pill: switching answering or
//! the integration off hands them back to Claude Code's own question.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, PoisonError};

use crate::activity_sources::claude_code::claude_code_status_message::ClaudeCodeStatusMessage;

#[derive(Clone)]
pub struct ClaudeCodePermissionAnsweringSwitch {
    is_answering_on: Arc<AtomicBool>,
    /// The running status thread, if any: woken on a change, so requests already in the pill
    /// leave at once instead of at the next hook event.
    status_message_sender: Arc<Mutex<Option<Sender<ClaudeCodeStatusMessage>>>>,
    /// Written by the status thread after every change, read by the settings window.
    waiting_request_count: Arc<AtomicUsize>,
}

impl ClaudeCodePermissionAnsweringSwitch {
    pub fn new(is_answering_on: bool) -> Self {
        Self {
            is_answering_on: Arc::new(AtomicBool::new(is_answering_on)),
            status_message_sender: Arc::default(),
            waiting_request_count: Arc::default(),
        }
    }

    pub fn record_waiting_request_count(&self, waiting_request_count: usize) {
        self.waiting_request_count.store(waiting_request_count, Ordering::Relaxed);
    }

    pub fn waiting_request_count(&self) -> usize {
        self.waiting_request_count.load(Ordering::Relaxed)
    }

    pub fn is_answering_on(&self) -> bool {
        self.is_answering_on.load(Ordering::Relaxed)
    }

    pub fn change(&self, should_answering_be_on: bool) {
        self.is_answering_on.store(should_answering_be_on, Ordering::Relaxed);
        if let Some(status_message_sender) = self.status_message_sender.lock().unwrap_or_else(PoisonError::into_inner).as_ref() {
            // A stopped status thread has nothing left to release.
            let _ = status_message_sender.send(ClaudeCodeStatusMessage::PermissionAnsweringChanged);
        }
    }

    /// Called by each newly started source; replaces the sender of the one before.
    pub fn connect_status_thread(&self, status_message_sender: Sender<ClaudeCodeStatusMessage>) {
        *self.status_message_sender.lock().unwrap_or_else(PoisonError::into_inner) = Some(status_message_sender);
    }
}
