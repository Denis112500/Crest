import { CLAUDE_CODE_ACTIVITY_KIND, CLAUDE_CODE_DISMISS_SESSION_ACTION_PREFIX } from "../../ipc/ipcChannelNames";
import { requestActivityAction } from "../../ipc/requestActivityAction";
import type { ClaudeCodeSessionRow } from "./claudeCodeStatusTypes";
import { describeClaudeCodeSessionStatus } from "./describeClaudeCodeSessionStatus";

const UNKNOWN_PROJECT_FOLDER_TEXT = "Unknown folder";
const DISMISS_BUTTON_TEXT = "×";

// One session in the open pill: a dot coloured by its status, its project folder and what it is
// doing, and for a finished one a "×" that removes it from the list (shown while the row is
// pointed at). The row itself isn't clickable: hook events don't say which window a session runs in.
export function createClaudeCodeSessionRowElement(claudeCodeSession: ClaudeCodeSessionRow): HTMLElement {
  const sessionRowElement = document.createElement("div");
  sessionRowElement.className = "claude-code-session-row";
  sessionRowElement.dataset.sessionStatus = claudeCodeSession.sessionStatus;
  const statusDotElement = document.createElement("span");
  statusDotElement.className = "claude-code-session-status-dot";
  // textContent, never innerHTML: folder names and commands must not be able to inject markup.
  const projectFolderElement = document.createElement("span");
  projectFolderElement.className = "claude-code-session-project";
  projectFolderElement.textContent = claudeCodeSession.projectFolderName || UNKNOWN_PROJECT_FOLDER_TEXT;
  const sessionStatusElement = document.createElement("span");
  sessionStatusElement.className = "claude-code-session-status";
  sessionStatusElement.textContent = describeClaudeCodeSessionStatus(claudeCodeSession);
  sessionRowElement.append(statusDotElement, projectFolderElement, sessionStatusElement);
  if (claudeCodeSession.canDismiss) {
    sessionRowElement.append(createDismissButton(claudeCodeSession, projectFolderElement.textContent));
  }
  return sessionRowElement;
}

function createDismissButton(claudeCodeSession: ClaudeCodeSessionRow, projectFolderText: string): HTMLButtonElement {
  const dismissButton = document.createElement("button");
  dismissButton.type = "button";
  dismissButton.className = "claude-code-session-dismiss-button";
  dismissButton.textContent = DISMISS_BUTTON_TEXT;
  dismissButton.setAttribute("aria-label", `Dismiss ${projectFolderText}`);
  dismissButton.title = "Dismiss";
  dismissButton.addEventListener("click", () => {
    // One click is enough: the row leaves with the next arrangement update.
    dismissButton.disabled = true;
    const dismissAction = `${CLAUDE_CODE_DISMISS_SESSION_ACTION_PREFIX}${claudeCodeSession.sessionId}`;
    requestActivityAction(CLAUDE_CODE_ACTIVITY_KIND, dismissAction).catch((actionError: unknown) => {
      console.error("Crest could not dismiss the Claude Code session:", actionError);
    });
  });
  return dismissButton;
}
