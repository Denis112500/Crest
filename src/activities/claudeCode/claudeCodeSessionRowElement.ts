import type { ClaudeCodeSessionRow } from "./claudeCodeStatusTypes";
import { describeClaudeCodeSessionStatus } from "./describeClaudeCodeSessionStatus";

const UNKNOWN_PROJECT_FOLDER_TEXT = "Unknown folder";

// One session in the open pill: a dot coloured by its status, its project folder and what it is
// doing. Not clickable: hook events don't say which window a session runs in.
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
  return sessionRowElement;
}
