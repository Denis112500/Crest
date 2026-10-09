import "../../styles/companionSegmentViews.css";

import { createSvgIconElement } from "../createSvgIconElement";
import type { ClaudeCodeSessionStatus, ClaudeCodeStatusPayload } from "./claudeCodeStatusTypes";
import { CLAUDE_CODE_TERMINAL_ICON_PATH } from "./claudeCodeTerminalIconPath";

// One or two words, because the segment is narrow; the open pill says the rest.
const COMPANION_WORD_BY_SESSION_STATUS: Record<ClaudeCodeSessionStatus, string> = {
  working: "Working",
  needsPermission: "Needs OK",
  waitingForInput: "Waiting",
  done: "Done",
};

export interface CompanionClaudeCodeView {
  companionViewElement: HTMLElement;
  showClaudeCodeStatus(claudeCodeStatus: ClaudeCodeStatusPayload): void;
}

// Claude Code next to the music: the terminal mark, coloured by the most urgent session's
// status (as the dots in the open list), and a word for it.
export function createCompanionClaudeCodeView(): CompanionClaudeCodeView {
  const companionViewElement = document.createElement("div");
  companionViewElement.className = "companion-segment-view companion-claude-code-view";
  const terminalIconElement = createSvgIconElement(CLAUDE_CODE_TERMINAL_ICON_PATH);
  terminalIconElement.classList.add("companion-claude-code-icon");
  const statusWordElement = document.createElement("span");
  statusWordElement.className = "companion-segment-text";
  companionViewElement.append(terminalIconElement, statusWordElement);

  return {
    companionViewElement,
    showClaudeCodeStatus(claudeCodeStatus) {
      const topSessionStatus = claudeCodeStatus.listedSessions[0].sessionStatus;
      companionViewElement.dataset.sessionStatus = topSessionStatus;
      statusWordElement.textContent = COMPANION_WORD_BY_SESSION_STATUS[topSessionStatus];
    },
  };
}
