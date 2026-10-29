//! The steps of a session's latest turn, shown when its row is unfolded (CC.3): the prompt it
//! started with, each tool call of the session itself, and each subagent it started, running
//! until their outcome arrives. Pure and unit-tested.

use std::collections::VecDeque;

use serde::Serialize;

use crate::backend_constants::CLAUDE_CODE_TURN_STEP_MAX_COUNT;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ClaudeCodeStepOutcome {
    Running,
    Done,
    Failed,
    /// Still running when the turn was cancelled, denied or stopped on an error: Claude Code
    /// sends no outcome for a cancelled tool (verified from docs).
    Stopped,
}

/// One step as the pill shows it (mirrored in `claudeCodeStatusTypes.ts`).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCodeTurnStep {
    /// The tool line ("Editing notes.md", "Running cargo test") or "Subagent: Explore".
    pub step_text: String,
    pub step_outcome: ClaudeCodeStepOutcome,
}

struct RecordedStep {
    /// The tool call's `tool_use_id`, or the subagent's `agent_id`; `None` from an older Claude
    /// Code that sends no id.
    step_id: Option<String>,
    is_subagent: bool,
    turn_step: ClaudeCodeTurnStep,
}

#[derive(Default)]
pub struct ClaudeCodeTurnSteps {
    /// The prompt's first line (`claude_code_turn_prompt_line.rs`).
    turn_prompt: Option<String>,
    /// Oldest first.
    recorded_steps: VecDeque<RecordedStep>,
}

impl ClaudeCodeTurnSteps {
    /// A new prompt starts a new turn, and its steps replace the last turn's.
    pub fn start_turn(&mut self, turn_prompt: Option<String>) {
        self.turn_prompt = turn_prompt;
        self.recorded_steps.clear();
    }

    /// A step whose id is already listed (a resumed subagent) runs again in its place.
    pub fn record_step_start(&mut self, step_id: Option<&str>, step_text: String, is_subagent: bool) {
        let running_step = ClaudeCodeTurnStep { step_text, step_outcome: ClaudeCodeStepOutcome::Running };
        if let Some(listed_step) = step_id.and_then(|step_id| self.find_step(step_id)) {
            listed_step.turn_step = running_step;
            return;
        }
        if self.recorded_steps.len() == CLAUDE_CODE_TURN_STEP_MAX_COUNT {
            self.recorded_steps.pop_front();
        }
        self.recorded_steps.push_back(RecordedStep { step_id: step_id.map(str::to_string), is_subagent, turn_step: running_step });
    }

    /// Finds the step by its id; an event without one (an older Claude Code) closes the newest
    /// step still running. An outcome for a step that already dropped out changes nothing.
    pub fn record_step_outcome(&mut self, step_id: Option<&str>, step_outcome: ClaudeCodeStepOutcome) {
        let finished_step = match step_id {
            Some(step_id) => self.find_step(step_id).filter(|listed_step| is_running(listed_step)),
            None => self.recorded_steps.iter_mut().rev().find(|listed_step| is_running(listed_step)),
        };
        if let Some(listed_step) = finished_step {
            listed_step.turn_step.step_outcome = step_outcome;
        }
    }

    /// The turn ended without its work done: whatever still runs won't report back.
    pub fn stop_running_steps(&mut self) {
        self.stop_running_steps_where(|_| true);
    }

    /// The user stopped the background subagents; the session's own tool may still be running.
    pub fn stop_running_subagent_steps(&mut self) {
        self.stop_running_steps_where(|listed_step| listed_step.is_subagent);
    }

    pub fn turn_prompt(&self) -> Option<String> {
        self.turn_prompt.clone()
    }

    /// Oldest first.
    pub fn list_steps(&self) -> Vec<ClaudeCodeTurnStep> {
        self.recorded_steps.iter().map(|listed_step| listed_step.turn_step.clone()).collect()
    }

    fn find_step(&mut self, step_id: &str) -> Option<&mut RecordedStep> {
        self.recorded_steps.iter_mut().rev().find(|listed_step| listed_step.step_id.as_deref() == Some(step_id))
    }

    fn stop_running_steps_where(&mut self, is_wanted: impl Fn(&RecordedStep) -> bool) {
        for listed_step in self.recorded_steps.iter_mut().filter(|listed_step| is_running(listed_step) && is_wanted(listed_step)) {
            listed_step.turn_step.step_outcome = ClaudeCodeStepOutcome::Stopped;
        }
    }
}

fn is_running(listed_step: &RecordedStep) -> bool {
    listed_step.turn_step.step_outcome == ClaudeCodeStepOutcome::Running
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step_outcomes(turn_steps: &ClaudeCodeTurnSteps) -> Vec<ClaudeCodeStepOutcome> {
        turn_steps.list_steps().into_iter().map(|turn_step| turn_step.step_outcome).collect()
    }

    #[test]
    fn outcomes_find_their_step_by_id_even_out_of_order() {
        let mut turn_steps = ClaudeCodeTurnSteps::default();
        turn_steps.record_step_start(Some("toolu_a"), "Reading a.txt".to_string(), false);
        turn_steps.record_step_start(Some("toolu_b"), "Reading b.txt".to_string(), false);
        turn_steps.record_step_outcome(Some("toolu_b"), ClaudeCodeStepOutcome::Failed);
        assert_eq!(step_outcomes(&turn_steps), [ClaudeCodeStepOutcome::Running, ClaudeCodeStepOutcome::Failed]);
        turn_steps.record_step_outcome(Some("toolu_a"), ClaudeCodeStepOutcome::Done);
        turn_steps.record_step_outcome(Some("toolu_unknown"), ClaudeCodeStepOutcome::Failed);
        assert_eq!(step_outcomes(&turn_steps), [ClaudeCodeStepOutcome::Done, ClaudeCodeStepOutcome::Failed]);
    }

    #[test]
    fn without_an_id_the_newest_running_step_is_closed() {
        let mut turn_steps = ClaudeCodeTurnSteps::default();
        turn_steps.record_step_start(None, "Searching the code".to_string(), false);
        turn_steps.record_step_start(None, "Running cargo test".to_string(), false);
        turn_steps.record_step_outcome(None, ClaudeCodeStepOutcome::Done);
        assert_eq!(step_outcomes(&turn_steps), [ClaudeCodeStepOutcome::Running, ClaudeCodeStepOutcome::Done]);
    }

    #[test]
    fn only_the_latest_steps_of_the_latest_turn_are_kept() {
        let mut turn_steps = ClaudeCodeTurnSteps::default();
        for step_number in 0..CLAUDE_CODE_TURN_STEP_MAX_COUNT + 2 {
            turn_steps.record_step_start(None, format!("Step {step_number}"), false);
        }
        let kept_steps = turn_steps.list_steps();
        assert_eq!(kept_steps.len(), CLAUDE_CODE_TURN_STEP_MAX_COUNT);
        assert_eq!(kept_steps[0].step_text, "Step 2");
        turn_steps.start_turn(Some("Next question".to_string()));
        assert!(turn_steps.list_steps().is_empty());
        assert_eq!(turn_steps.turn_prompt().as_deref(), Some("Next question"));
    }

    #[test]
    fn a_resumed_subagent_runs_again_in_its_place() {
        let mut turn_steps = ClaudeCodeTurnSteps::default();
        turn_steps.record_step_start(Some("agent_1"), "Subagent: Explore".to_string(), true);
        turn_steps.record_step_outcome(Some("agent_1"), ClaudeCodeStepOutcome::Done);
        turn_steps.record_step_start(Some("toolu_a"), "Editing main.rs".to_string(), false);
        turn_steps.record_step_start(Some("agent_1"), "Subagent: Explore".to_string(), true);
        assert_eq!(turn_steps.list_steps().len(), 2);
        assert_eq!(step_outcomes(&turn_steps), [ClaudeCodeStepOutcome::Running, ClaudeCodeStepOutcome::Running]);
    }

    #[test]
    fn stopping_closes_only_what_still_runs_and_subagents_can_stop_alone() {
        let mut turn_steps = ClaudeCodeTurnSteps::default();
        turn_steps.record_step_start(Some("toolu_a"), "Reading a.txt".to_string(), false);
        turn_steps.record_step_outcome(Some("toolu_a"), ClaudeCodeStepOutcome::Done);
        turn_steps.record_step_start(Some("agent_1"), "Subagent: Explore".to_string(), true);
        turn_steps.record_step_start(Some("toolu_b"), "Running cargo test".to_string(), false);
        turn_steps.stop_running_subagent_steps();
        assert_eq!(
            step_outcomes(&turn_steps),
            [ClaudeCodeStepOutcome::Done, ClaudeCodeStepOutcome::Stopped, ClaudeCodeStepOutcome::Running]
        );
        turn_steps.stop_running_steps();
        assert_eq!(
            step_outcomes(&turn_steps),
            [ClaudeCodeStepOutcome::Done, ClaudeCodeStepOutcome::Stopped, ClaudeCodeStepOutcome::Stopped]
        );
    }
}
