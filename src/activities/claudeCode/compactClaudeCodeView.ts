import "../../styles/compactClaudeCodeView.css";

import { createSvgIconElement } from "../createSvgIconElement";
import type { ClaudeCodeStatusPayload } from "./claudeCodeStatusTypes";
import { CLAUDE_CODE_TERMINAL_ICON_PATH } from "./claudeCodeTerminalIconPath";
import { describeClaudeCodeStatus } from "./describeClaudeCodeStatus";

export interface CompactClaudeCodeView {
  compactViewElement: HTMLElement;
  showClaudeCodeStatus(claudeCodeStatus: ClaudeCodeStatusPayload): void;
}

// The small pill: a terminal mark, what the session is doing, and "+N" for other sessions.
export function createCompactClaudeCodeView(): CompactClaudeCodeView {
  const compactViewElement = document.createElement("div");
  compactViewElement.className = "compact-claude-code-view";
  const terminalIconElement = createSvgIconElement(CLAUDE_CODE_TERMINAL_ICON_PATH);
  terminalIconElement.classList.add("claude-code-terminal-icon");
  const compactStatusElement = document.createElement("span");
  compactStatusElement.className = "compact-claude-code-status";
  const otherSessionCountElement = document.createElement("span");
  otherSessionCountElement.className = "claude-code-other-session-count";
  compactViewElement.append(terminalIconElement, compactStatusElement, otherSessionCountElement);

  return {
    compactViewElement,
    showClaudeCodeStatus(claudeCodeStatus) {
      // textContent, never innerHTML: file names and commands must not be able to inject markup.
      compactStatusElement.textContent = describeClaudeCodeStatus(claudeCodeStatus);
      otherSessionCountElement.textContent =
        claudeCodeStatus.otherSessionCount > 0 ? `+${claudeCodeStatus.otherSessionCount}` : "";
    },
  };
}
