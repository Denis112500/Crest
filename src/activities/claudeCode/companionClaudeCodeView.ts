import "../../styles/companionMarkViews.css";

import { createSvgIconElement } from "../createSvgIconElement";
import type { ClaudeCodeStatusPayload } from "./claudeCodeStatusTypes";
import { CLAUDE_CODE_TERMINAL_ICON_PATH } from "./claudeCodeTerminalIconPath";
import { describeClaudeCodeSessionStatus } from "./describeClaudeCodeSessionStatus";

export interface CompanionClaudeCodeView {
  companionViewElement: HTMLElement;
  showClaudeCodeStatus(claudeCodeStatus: ClaudeCodeStatusPayload): void;
}

// Claude Code next to the music: only the terminal mark, coloured by the most urgent session's
// status (as the dots in the open list). The status in words is in its tooltip and accessible
// name, since the pill no longer has room for a word (milestone F).
export function createCompanionClaudeCodeView(): CompanionClaudeCodeView {
  const companionViewElement = document.createElement("div");
  companionViewElement.className = "companion-mark-view companion-claude-code-view";
  companionViewElement.setAttribute("role", "img");
  const terminalIconElement = createSvgIconElement(CLAUDE_CODE_TERMINAL_ICON_PATH);
  terminalIconElement.classList.add("companion-claude-code-icon");
  companionViewElement.append(terminalIconElement);

  return {
    companionViewElement,
    showClaudeCodeStatus(claudeCodeStatus) {
      const topSession = claudeCodeStatus.listedSessions[0];
      companionViewElement.dataset.sessionStatus = topSession.sessionStatus;
      const claudeCodeCompanionDescription = `Claude Code: ${describeClaudeCodeSessionStatus(topSession)}`;
      companionViewElement.title = claudeCodeCompanionDescription;
      companionViewElement.setAttribute("aria-label", claudeCodeCompanionDescription);
    },
  };
}
