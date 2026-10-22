//! The subagents one Claude Code session is running, by `agent_id`: added on `SubagentStart` or
//! on any tool event a subagent sends, removed on `SubagentStop`. Whether Claude Code sends
//! `SubagentStop` for a subagent killed by Esc is not documented, so one that sends nothing for
//! the session's silence timeout is dropped too.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::activity_sources::claude_code::claude_code_tool_summary::summarize_running_subagents;

#[derive(Default)]
pub struct ClaudeCodeRunningSubagents {
    last_sign_of_life_by_agent_id: HashMap<String, Instant>,
}

impl ClaudeCodeRunningSubagents {
    /// A subagent started or did something: it runs.
    pub fn record_sign_of_life(&mut self, agent_id: &str, event_time: Instant) {
        self.last_sign_of_life_by_agent_id.insert(agent_id.to_string(), event_time);
    }

    pub fn record_stop(&mut self, agent_id: &str) {
        self.last_sign_of_life_by_agent_id.remove(agent_id);
    }

    /// The session's turn was interrupted: its subagents went with it. One still running in the
    /// background comes back with its next tool event.
    pub fn forget_all(&mut self) {
        self.last_sign_of_life_by_agent_id.clear();
    }

    pub fn forget_silent_for(&mut self, silence_timeout: Duration, current_time: Instant) {
        self.last_sign_of_life_by_agent_id
            .retain(|_, last_sign_of_life| current_time.duration_since(*last_sign_of_life) < silence_timeout);
    }

    /// When the quietest subagent reaches the silence timeout, if any runs.
    pub fn next_silence_deadline(&self, silence_timeout: Duration) -> Option<Instant> {
        self.last_sign_of_life_by_agent_id.values().map(|last_sign_of_life| *last_sign_of_life + silence_timeout).min()
    }

    /// The session row's line and the subagents it doesn't mention. While the main tool call is
    /// the one that started them, the line counts them ("Running 2 subagents"); otherwise the line
    /// stays as it is and they show next to it ("· 1 subagent").
    pub fn describe_with_line(&self, tool_summary: Option<&str>, is_tool_a_subagent_launch: bool) -> (Option<String>, usize) {
        let running_subagent_count = self.last_sign_of_life_by_agent_id.len();
        if is_tool_a_subagent_launch && running_subagent_count > 0 {
            (Some(summarize_running_subagents(running_subagent_count)), 0)
        } else {
            (tool_summary.map(str::to_string), running_subagent_count)
        }
    }

    pub fn running_count(&self) -> usize {
        self.last_sign_of_life_by_agent_id.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subagents_count_until_they_stop_or_fall_silent() {
        let (mut running_subagents, start_time) = (ClaudeCodeRunningSubagents::default(), Instant::now());
        let silence_timeout = Duration::from_secs(600);
        running_subagents.record_sign_of_life("explore-1", start_time);
        running_subagents.record_sign_of_life("explore-2", start_time + Duration::from_secs(60));
        running_subagents.record_sign_of_life("explore-2", start_time + Duration::from_secs(120));
        assert_eq!(running_subagents.running_count(), 2);
        assert_eq!(running_subagents.next_silence_deadline(silence_timeout), Some(start_time + silence_timeout));
        running_subagents.forget_silent_for(silence_timeout, start_time + silence_timeout);
        assert_eq!(running_subagents.running_count(), 1);
        running_subagents.record_stop("explore-2");
        assert_eq!((running_subagents.running_count(), running_subagents.next_silence_deadline(silence_timeout)), (0, None));
    }
}
