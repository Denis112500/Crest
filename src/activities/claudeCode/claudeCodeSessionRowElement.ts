import { CLAUDE_CODE_ACTIVITY_KIND, CLAUDE_CODE_DISMISS_SESSION_ACTION_PREFIX } from "../../ipc/ipcChannelNames";
import { requestActivityAction } from "../../ipc/requestActivityAction";
import type { ClaudeCodeSessionRow } from "./claudeCodeStatusTypes";
import { describeClaudeCodeSessionStatus } from "./describeClaudeCodeSessionStatus";

const UNKNOWN_PROJECT_FOLDER_TEXT = "Unknown folder";
const SESSION_TITLE_SEPARATOR = "· ";
const DISMISS_BUTTON_TEXT = "×";

// One session in the open pill: a dot coloured by its status, its project folder and, dimmed, the
// session's title (so sessions of one folder can be told apart; look A, CC.2d), what it is doing
// at the right end, and for a finished one a "×" that removes it from the list (shown while the
// session is pointed at). A done session adds a dim line from Claude's final answer under it.
// The row itself isn't clickable: hook events don't say which window a session runs in.
export function createClaudeCodeSessionRowElement(claudeCodeSession: ClaudeCodeSessionRow): HTMLElement {
  const sessionEntryElement = document.createElement("div");
  sessionEntryElement.className = "claude-code-session-entry";
  sessionEntryElement.append(createSessionLineElement(claudeCodeSession));
  if (claudeCodeSession.turnSummary !== null) {
    const turnSummaryElement = document.createElement("div");
    turnSummaryElement.className = "claude-code-session-turn-summary";
    // textContent, never innerHTML: Claude's answer must not be able to inject markup.
    turnSummaryElement.textContent = claudeCodeSession.turnSummary;
    turnSummaryElement.title = claudeCodeSession.turnSummary;
    sessionEntryElement.append(turnSummaryElement);
  }
  return sessionEntryElement;
}

function createSessionLineElement(claudeCodeSession: ClaudeCodeSessionRow): HTMLElement {
  const sessionRowElement = document.createElement("div");
  sessionRowElement.className = "claude-code-session-row";
  sessionRowElement.dataset.sessionStatus = claudeCodeSession.sessionStatus;
  const statusDotElement = document.createElement("span");
  statusDotElement.className = "claude-code-session-status-dot";
  const sessionNameElement = createSessionNameElement(claudeCodeSession);
  const sessionStatusElement = document.createElement("span");
  sessionStatusElement.className = "claude-code-session-status";
  sessionStatusElement.textContent = describeClaudeCodeSessionStatus(claudeCodeSession);
  sessionRowElement.append(statusDotElement, sessionNameElement, sessionStatusElement);
  if (claudeCodeSession.canDismiss) {
    sessionRowElement.append(createDismissButton(claudeCodeSession, sessionNameElement.title));
  }
  return sessionRowElement;
}

function createSessionNameElement(claudeCodeSession: ClaudeCodeSessionRow): HTMLElement {
  const sessionNameElement = document.createElement("span");
  sessionNameElement.className = "claude-code-session-name";
  // textContent, never innerHTML: folder names, titles and commands must not be able to inject markup.
  const projectFolderElement = document.createElement("span");
  projectFolderElement.className = "claude-code-session-project";
  projectFolderElement.textContent = claudeCodeSession.projectFolderName || UNKNOWN_PROJECT_FOLDER_TEXT;
  sessionNameElement.append(projectFolderElement);
  sessionNameElement.title = projectFolderElement.textContent;
  if (claudeCodeSession.sessionTitle !== null) {
    const sessionTitleElement = document.createElement("span");
    sessionTitleElement.className = "claude-code-session-title";
    sessionTitleElement.textContent = `${SESSION_TITLE_SEPARATOR}${claudeCodeSession.sessionTitle}`;
    sessionNameElement.append(sessionTitleElement);
    sessionNameElement.title = `${projectFolderElement.textContent} ${sessionTitleElement.textContent}`;
  }
  return sessionNameElement;
}

function createDismissButton(claudeCodeSession: ClaudeCodeSessionRow, sessionNameText: string): HTMLButtonElement {
  const dismissButton = document.createElement("button");
  dismissButton.type = "button";
  dismissButton.className = "claude-code-session-dismiss-button";
  dismissButton.textContent = DISMISS_BUTTON_TEXT;
  dismissButton.setAttribute("aria-label", `Dismiss ${sessionNameText}`);
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
