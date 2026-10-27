//! One Claude Code session as the tracker keeps it: its fields, the bookkeeping the tracker asks
//! for (news, title, usage limit, silence) and the status rules its other parts share. Split from
//! `claude_code_session_tracker.rs` (code review before 0.4.0), which keeps the sessions, their
//! order, the news count and the timeouts; how events change it lives next to this file.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::activity_sources::claude_code::claude_code_running_subagents::ClaudeCodeRunningSubagents;
use crate::activity_sources::claude_code::claude_code_session_list::ClaudeCodeSessionStatus;
use crate::activity_sources::claude_code::claude_code_session_title::ClaudeCodeSessionTitle;
use crate::activity_sources::claude_code::claude_code_turn_steps::ClaudeCodeTurnSteps;
use crate::activity_sources::claude_code::claude_code_usage_limit_wait::ClaudeCodeUsageLimitWait;

pub struct TrackedSession {
    pub(super) project_folder_name: String,
    pub transcript_path: Option<PathBuf>,
    /// `None` right after `SessionStart`: an idle new session has nothing to show yet.
    pub(super) session_status: Option<ClaudeCodeSessionStatus>,
    pub(super) tool_summary: Option<String>,
    /// The main tool call starts subagents, so the line counts them instead of naming the tool.
    pub(super) is_tool_a_subagent_launch: bool,
    pub(super) running_subagents: ClaudeCodeRunningSubagents,
    pub(super) usage_limit_wait: Option<ClaudeCodeUsageLimitWait>,
    /// How the turn ended, shown under the row: a line from Claude's answer (`Stop`) or the API
    /// error that stopped it (`StopFailure`); any later status forgets it.
    pub(super) turn_summary: Option<String>,
    /// The latest turn's tool calls, shown when the row is unfolded; kept after the turn ends.
    pub(super) turn_steps: ClaudeCodeTurnSteps,
    pub(super) session_title: Option<ClaudeCodeSessionTitle>,
    /// The transcript's start was searched for a title (once per session).
    pub(super) is_title_looked_up: bool,
    /// The tracker's news count right after this session's latest news (0: none yet).
    pub(super) news_count: u64,
    pub(super) last_event_time: Instant,
    pub(super) last_event_order: u64,
}

impl TrackedSession {
    /// Crest may start in the middle of a session, so any event can be the first one seen.
    pub fn new(event_time: Instant, event_order: u64) -> Self {
        Self {
            project_folder_name: String::new(),
            transcript_path: None,
            session_status: None,
            tool_summary: None,
            is_tool_a_subagent_launch: false,
            running_subagents: ClaudeCodeRunningSubagents::default(),
            usage_limit_wait: None,
            turn_summary: None,
            turn_steps: ClaudeCodeTurnSteps::default(),
            session_title: None,
            is_title_looked_up: false,
            news_count: 0,
            last_event_time: event_time,
            last_event_order: event_order,
        }
    }

    /// The turn ended in a way that already tells the user what happened.
    pub(super) fn has_turn_ending_status(&self) -> bool {
        matches!(
            self.session_status,
            Some(
                ClaudeCodeSessionStatus::Done
                    | ClaudeCodeSessionStatus::Errored
                    | ClaudeCodeSessionStatus::Cancelled
                    | ClaudeCodeSessionStatus::Denied
                    | ClaudeCodeSessionStatus::UsageLimited
            )
        )
    }

    /// The turn has ended, but subagents it started in the background still run.
    pub(super) fn is_waiting_on_subagents(&self) -> bool {
        self.session_status == Some(ClaudeCodeSessionStatus::Done) && self.running_subagents.running_count() > 0
    }

    /// So the light can tell this session's news from older news the user has already seen.
    pub fn note_news(&mut self, news_count: u64) {
        self.news_count = news_count;
    }

    /// The transcript whose start may hold the title of a session met mid-way: given once, as soon
    /// as the transcript is known, and only while the session has no title.
    pub fn take_title_lookup_transcript(&mut self) -> Option<PathBuf> {
        if self.is_title_looked_up || self.session_title.is_some() {
            return None;
        }
        let transcript_path = self.transcript_path.clone()?;
        self.is_title_looked_up = true;
        Some(transcript_path)
    }

    /// From the hook (a chosen title) or the transcript watch; not news, and it doesn't count as
    /// the session's latest event.
    pub fn show_session_title(&mut self, session_title: ClaudeCodeSessionTitle) {
        if session_title.replaces(self.session_title.as_ref()) {
            self.session_title = Some(session_title);
        }
    }

    /// The reset time read from the transcript after the session stopped on a usage limit.
    pub fn show_usage_limit_reset(&mut self, resets_at_unix_seconds: u64, time_until_reset: Duration, current_time: Instant) {
        if self.session_status == Some(ClaudeCodeSessionStatus::UsageLimited) {
            self.usage_limit_wait = Some(ClaudeCodeUsageLimitWait::new(resets_at_unix_seconds, time_until_reset, current_time));
        }
    }

    /// Working (its own turn or only its subagents), or waiting for the user's OK.
    pub fn is_busy(&self) -> bool {
        self.session_status.is_some_and(ClaudeCodeSessionStatus::is_busy) || self.is_waiting_on_subagents()
    }

    /// When the session goes for lack of news: the silence timeout after its last event, or after
    /// the reset of a usage limit it waits for, whichever is later.
    pub fn silence_deadline(&self, silence_timeout: Duration) -> Instant {
        let after_last_event = self.last_event_time + silence_timeout;
        match self.usage_limit_wait {
            Some(usage_limit_wait) => usage_limit_wait.silence_deadline(after_last_event, silence_timeout),
            None => after_last_event,
        }
    }

    pub fn next_subagent_silence_deadline(&self, silence_timeout: Duration) -> Option<Instant> {
        self.running_subagents.next_silence_deadline(silence_timeout)
    }

    pub fn forget_subagents_silent_for(&mut self, silence_timeout: Duration, current_time: Instant) {
        self.running_subagents.forget_silent_for(silence_timeout, current_time);
    }

    /// For the silence timeout (time) and the list's "newest first" (order).
    pub(super) fn note_event(&mut self, event_time: Instant, event_order: u64) {
        self.last_event_time = event_time;
        self.last_event_order = event_order;
    }

    /// Leaving the usage-limited state also forgets when the limit resets, and any new status
    /// forgets the last turn's summary (a `Stop` sets its own afterwards).
    pub(super) fn show_status_without_tool(&mut self, session_status: ClaudeCodeSessionStatus) {
        if session_status != ClaudeCodeSessionStatus::UsageLimited {
            self.usage_limit_wait = None;
        }
        self.turn_summary = None;
        self.session_status = Some(session_status);
        self.tool_summary = None;
        self.is_tool_a_subagent_launch = false;
    }
}
