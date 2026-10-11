import { CLAUDE_CODE_PERMISSION_BUTTON_ARMING_DELAY_MILLISECONDS } from "../../frontendConstants";
import {
  CLAUDE_CODE_ACTIVITY_KIND,
  CLAUDE_CODE_ALLOW_PERMISSION_ACTION_PREFIX,
  CLAUDE_CODE_DENY_PERMISSION_ACTION_PREFIX,
} from "../../ipc/ipcChannelNames";
import { requestActivityAction } from "../../ipc/requestActivityAction";

const DENY_BUTTON_TEXT = "Deny";
const ALLOW_BUTTON_TEXT = "Allow";
const READ_TO_THE_END_BUTTON_TEXT = "Read to the end";
const TOO_LONG_TO_SHOW_NOTE_TEXT = "Answer in Claude Code";

export interface ClaudeCodePermissionAnswerButtons {
  buttonRowElement: HTMLElement;
  /** A new request: nothing answered yet. A cut command gets a note instead of Allow. */
  prepareForRequest(requestId: number, isCommandCut: boolean): void;
  /** Disables both buttons for a moment (the alert became visible, or a new request). */
  armAfterDelay(): void;
  /** The request was scrolled to the end: Allow may become clickable. */
  refreshButtons(): void;
}

// Deny and Allow for the shown request. Clicks count only after a short pause, only once per
// request (a second click could reach the next one), and Allow only once the whole request has
// been seen; a command too long to send can only be denied here.
export function createClaudeCodePermissionAnswerButtons(
  hasWholeRequestBeenSeen: () => boolean,
): ClaudeCodePermissionAnswerButtons {
  const buttonRowElement = document.createElement("div");
  buttonRowElement.className = "claude-code-permission-buttons";
  let shownRequestId: number | null = null;
  let isCommandCut = false;
  let areButtonsArmed = false;
  let wasAnswerSent = false;
  let buttonArmingTimer: number | undefined;

  const refreshButtons = (): void => {
    const canAnswerNow = areButtonsArmed && !wasAnswerSent;
    const isWholeRequestSeen = hasWholeRequestBeenSeen();
    denyButton.disabled = !canAnswerNow;
    allowButton.disabled = !canAnswerNow || !isWholeRequestSeen;
    allowButton.textContent = isWholeRequestSeen ? ALLOW_BUTTON_TEXT : READ_TO_THE_END_BUTTON_TEXT;
    allowButton.classList.toggle("is-allow", isWholeRequestSeen);
    allowButton.hidden = isCommandCut;
    tooLongNoteElement.hidden = !isCommandCut;
  };

  const createAnswerButton = (buttonText: string, actionPrefix: string): HTMLButtonElement => {
    const answerButton = document.createElement("button");
    answerButton.type = "button";
    answerButton.className = "claude-code-permission-button";
    answerButton.textContent = buttonText;
    answerButton.addEventListener("click", () => {
      if (shownRequestId === null) {
        return;
      }
      window.clearTimeout(buttonArmingTimer);
      wasAnswerSent = true;
      refreshButtons();
      const permissionAction = `${actionPrefix}${shownRequestId}`;
      requestActivityAction(CLAUDE_CODE_ACTIVITY_KIND, permissionAction).catch((actionError: unknown) => {
        console.error(`Crest could not send "${permissionAction}" to Claude Code:`, actionError);
      });
    });
    return answerButton;
  };
  const denyButton = createAnswerButton(DENY_BUTTON_TEXT, CLAUDE_CODE_DENY_PERMISSION_ACTION_PREFIX);
  const allowButton = createAnswerButton(ALLOW_BUTTON_TEXT, CLAUDE_CODE_ALLOW_PERMISSION_ACTION_PREFIX);
  const tooLongNoteElement = document.createElement("span");
  tooLongNoteElement.className = "claude-code-permission-too-long-note";
  tooLongNoteElement.textContent = TOO_LONG_TO_SHOW_NOTE_TEXT;
  buttonRowElement.append(denyButton, allowButton, tooLongNoteElement);

  return {
    buttonRowElement,
    refreshButtons,
    prepareForRequest(requestId, isCommandCutNow) {
      shownRequestId = requestId;
      isCommandCut = isCommandCutNow;
      wasAnswerSent = false;
    },
    armAfterDelay() {
      window.clearTimeout(buttonArmingTimer);
      areButtonsArmed = false;
      refreshButtons();
      buttonArmingTimer = window.setTimeout(() => {
        areButtonsArmed = true;
        refreshButtons();
      }, CLAUDE_CODE_PERMISSION_BUTTON_ARMING_DELAY_MILLISECONDS);
    },
  };
}
