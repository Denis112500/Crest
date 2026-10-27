import type { ClaudeCodeTurnStep } from "./claudeCodeStatusTypes";

const NO_STEPS_TEXT = "No steps in this turn yet";
// The running step's mark is a ring drawn in CSS (claudeCodeSessionUnfolding.css).
const STEP_OUTCOME_MARKS: Record<ClaudeCodeTurnStep["stepOutcome"], string> = {
  running: "",
  done: "✓",
  failed: "✗",
};

// Under an unfolded row: every step of its latest turn, oldest first, in a box as tall as
// `visibleStepLineCount` lines (as many as the open pill has room for; expandedClaudeCodeView.ts
// counts them and scrolls the box to the newest step); the wheel scrolls to the older ones.
export function createClaudeCodeTurnStepsElement(turnSteps: ClaudeCodeTurnStep[], visibleStepLineCount: number): HTMLElement {
  const turnStepsElement = document.createElement("div");
  turnStepsElement.className = "claude-code-turn-steps";
  turnStepsElement.style.height = `calc(${visibleStepLineCount} * var(--claude-code-turn-step-height))`;
  if (turnSteps.length === 0) {
    turnStepsElement.append(createStepLineElement(NO_STEPS_TEXT, null));
    return turnStepsElement;
  }
  turnStepsElement.append(...turnSteps.map((turnStep) => createStepLineElement(turnStep.stepText, turnStep.stepOutcome)));
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
