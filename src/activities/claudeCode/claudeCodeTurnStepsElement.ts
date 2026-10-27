import "../../styles/claudeCodeTurnSteps.css";

import type { ClaudeCodeTurnStep } from "./claudeCodeStatusTypes";

const NO_STEPS_TEXT = "No steps in this turn yet";
const STEP_OUTCOME_MARKS: Record<ClaudeCodeTurnStep["stepOutcome"], string> = {
  running: "›",
  done: "✓",
  failed: "✗",
};

// Under an unfolded row: the newest steps of its latest turn, oldest of them first, as many as
// the open pill has room for (expandedClaudeCodeView.ts counts the lines).
export function createClaudeCodeTurnStepsElement(turnSteps: ClaudeCodeTurnStep[], shownStepLineCount: number): HTMLElement {
  const turnStepsElement = document.createElement("div");
  turnStepsElement.className = "claude-code-turn-steps";
  if (turnSteps.length === 0) {
    turnStepsElement.append(createStepLineElement(NO_STEPS_TEXT, null));
    return turnStepsElement;
  }
  const shownSteps = turnSteps.slice(-shownStepLineCount);
  turnStepsElement.append(...shownSteps.map((turnStep) => createStepLineElement(turnStep.stepText, turnStep.stepOutcome)));
  return turnStepsElement;
}

function createStepLineElement(stepText: string, stepOutcome: ClaudeCodeTurnStep["stepOutcome"] | null): HTMLElement {
  const stepLineElement = document.createElement("div");
  stepLineElement.className = "claude-code-turn-step";
  const stepMarkElement = document.createElement("span");
  stepMarkElement.className = "claude-code-turn-step-mark";
  stepMarkElement.textContent = stepOutcome === null ? "" : STEP_OUTCOME_MARKS[stepOutcome];
  const stepTextElement = document.createElement("span");
  stepTextElement.className = "claude-code-turn-step-text";
  // textContent, never innerHTML: commands and file names must not be able to inject markup.
  stepTextElement.textContent = stepText;
  stepLineElement.append(stepMarkElement, stepTextElement);
  if (stepOutcome !== null) {
    stepLineElement.dataset.stepOutcome = stepOutcome;
  }
  stepLineElement.title = stepText;
  return stepLineElement;
}
