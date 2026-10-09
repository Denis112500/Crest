import "../../styles/expandedClaudeCodeView.css";

import { createSvgIconElement } from "../createSvgIconElement";
import { createClaudeCodeSessionRowElement } from "./claudeCodeSessionRowElement";
import type { ClaudeCodeStatusPayload } from "./claudeCodeStatusTypes";
import { CLAUDE_CODE_TERMINAL_ICON_PATH } from "./claudeCodeTerminalIconPath";

const CLAUDE_CODE_PRODUCT_NAME = "Claude Code";

export interface ExpandedClaudeCodeView {
  expandedViewElement: HTMLElement;
  showClaudeCodeStatus(claudeCodeStatus: ClaudeCodeStatusPayload): void;
}

// The open pill: how many sessions there are, one row per listed session (most urgent first)
// and "+N more" for the rest. Its height is calculateClaudeCodeExpandedHeight.ts's job.
export function createExpandedClaudeCodeView(): ExpandedClaudeCodeView {
  const expandedViewElement = document.createElement("div");
  expandedViewElement.className = "expanded-claude-code-view";

  const headerElement = document.createElement("div");
  headerElement.className = "expanded-claude-code-header";
  const terminalIconElement = createSvgIconElement(CLAUDE_CODE_TERMINAL_ICON_PATH);
  terminalIconElement.classList.add("claude-code-terminal-icon", "expanded-claude-code-icon");
  const productNameElement = document.createElement("div");
  productNameElement.className = "expanded-claude-code-product";
  productNameElement.textContent = CLAUDE_CODE_PRODUCT_NAME;
  const sessionCountElement = document.createElement("div");
  sessionCountElement.className = "expanded-claude-code-session-count";
  headerElement.append(terminalIconElement, productNameElement, sessionCountElement);

  const sessionListElement = document.createElement("div");
  sessionListElement.className = "claude-code-session-list";
  const unlistedSessionsElement = document.createElement("div");
  unlistedSessionsElement.className = "claude-code-unlisted-sessions";
  expandedViewElement.append(headerElement, sessionListElement, unlistedSessionsElement);

  return {
    expandedViewElement,
    showClaudeCodeStatus(claudeCodeStatus) {
      const unlistedSessionCount = claudeCodeStatus.unlistedSessionCount;
      const sessionCount = claudeCodeStatus.listedSessions.length + unlistedSessionCount;
      sessionCountElement.textContent = `${sessionCount} session${sessionCount === 1 ? "" : "s"}`;
      sessionListElement.replaceChildren(...claudeCodeStatus.listedSessions.map(createClaudeCodeSessionRowElement));
      // Hidden, not just empty: a hidden element adds no gap, which the height calculation relies on.
      unlistedSessionsElement.hidden = unlistedSessionCount === 0;
      unlistedSessionsElement.textContent = `+${unlistedSessionCount} more`;
    },
  };
}
