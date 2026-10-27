//! The steps of a session's latest turn, shown when its row is unfolded (CC.3): each tool call,
//! running until its outcome arrives. Pure and unit-tested.

use std::collections::VecDeque;

use serde::Serialize;

use crate::backend_constants::CLAUDE_CODE_TURN_STEP_MAX_COUNT;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ClaudeCodeStepOutcome {
    Running,
    Done,
    Failed,
}

/// One step as the pill shows it (mirrored in `claudeCodeStatusTypes.ts`).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCodeTurnStep {
    /// The tool line, e.g. "Editing notes.md" or "Running cargo test".
    pub step_text: String,
    pub step_outcome: ClaudeCodeStepOutcome,
}

#[derive(Default)]
pub struct ClaudeCodeTurnSteps {
    /// Oldest first, each with its tool call's `tool_use_id` when the event had one.
    recorded_steps: VecDeque<(Option<String>, ClaudeCodeTurnStep)>,
}

impl ClaudeCodeTurnSteps {
    /// A new prompt starts a new turn, and its steps replace the last turn's.
    pub fn start_turn(&mut self) {
        self.recorded_steps.clear();
    }

    pub fn record_tool_start(&mut self, tool_use_id: Option<&str>, step_text: String) {
        if self.recorded_steps.len() == CLAUDE_CODE_TURN_STEP_MAX_COUNT {
            self.recorded_steps.pop_front();
        }
        let running_step = ClaudeCodeTurnStep { step_text, step_outcome: ClaudeCodeStepOutcome::Running };
        self.recorded_steps.push_back((tool_use_id.map(str::to_string), running_step));
    }

    /// Finds the step by its id; an event without one (an older Claude Code) closes the newest
    /// step still running. An outcome for a step that already dropped out changes nothing.
    pub fn record_tool_outcome(&mut self, tool_use_id: Option<&str>, step_outcome: ClaudeCodeStepOutcome) {
        let is_running = |recorded_step: &&mut (Option<String>, ClaudeCodeTurnStep)| {
            recorded_step.1.step_outcome == ClaudeCodeStepOutcome::Running
        };
        let finished_step = match tool_use_id {
            Some(tool_use_id) => self
                .recorded_steps
                .iter_mut()
                .rev()
                .filter(is_running)
                .find(|(recorded_id, _)| recorded_id.as_deref() == Some(tool_use_id)),
            None => self.recorded_steps.iter_mut().rev().find(is_running),
        };
        if let Some((_, turn_step)) = finished_step {
            turn_step.step_outcome = step_outcome;
        }
    }

    /// Oldest first.
    pub fn list_steps(&self) -> Vec<ClaudeCodeTurnStep> {
        self.recorded_steps.iter().map(|(_, turn_step)| turn_step.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step_outcomes(turn_steps: &ClaudeCodeTurnSteps) -> Vec<(String, ClaudeCodeStepOutcome)> {
        turn_steps.list_steps().into_iter().map(|turn_step| (turn_step.step_text, turn_step.step_outcome)).collect()
    }

    #[test]
    fn outcomes_find_their_step_by_id_even_out_of_order() {
        let mut turn_steps = ClaudeCodeTurnSteps::default();
        turn_steps.record_tool_start(Some("toolu_a"), "Reading a.txt".to_string());
        turn_steps.record_tool_start(Some("toolu_b"), "Reading b.txt".to_string());
        turn_steps.record_tool_outcome(Some("toolu_b"), ClaudeCodeStepOutcome::Failed);
        assert_eq!(
            step_outcomes(&turn_steps),
            vec![
                ("Reading a.txt".to_string(), ClaudeCodeStepOutcome::Running),
                ("Reading b.txt".to_string(), ClaudeCodeStepOutcome::Failed)
            ]
        );
        turn_steps.record_tool_outcome(Some("toolu_a"), ClaudeCodeStepOutcome::Done);
        turn_steps.record_tool_outcome(Some("toolu_unknown"), ClaudeCodeStepOutcome::Failed);
        assert_eq!(step_outcomes(&turn_steps)[0].1, ClaudeCodeStepOutcome::Done);
        assert_eq!(step_outcomes(&turn_steps)[1].1, ClaudeCodeStepOutcome::Failed);
    }

    #[test]
    fn without_an_id_the_newest_running_step_is_closed() {
        let mut turn_steps = ClaudeCodeTurnSteps::default();
        turn_steps.record_tool_start(None, "Searching the code".to_string());
        turn_steps.record_tool_start(None, "Running cargo test".to_string());
        turn_steps.record_tool_outcome(None, ClaudeCodeStepOutcome::Done);
        assert_eq!(step_outcomes(&turn_steps)[0].1, ClaudeCodeStepOutcome::Running);
        assert_eq!(step_outcomes(&turn_steps)[1].1, ClaudeCodeStepOutcome::Done);
    }

    #[test]
    fn only_the_latest_steps_of_the_latest_turn_are_kept() {
        let mut turn_steps = ClaudeCodeTurnSteps::default();
        for step_number in 0..CLAUDE_CODE_TURN_STEP_MAX_COUNT + 2 {
            turn_steps.record_tool_start(None, format!("Step {step_number}"));
        }
        let kept_steps = turn_steps.list_steps();
        assert_eq!(kept_steps.len(), CLAUDE_CODE_TURN_STEP_MAX_COUNT);
        assert_eq!(kept_steps[0].step_text, "Step 2");
        turn_steps.start_turn();
        assert!(turn_steps.list_steps().is_empty());
    }
}
