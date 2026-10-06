import "../../styles/expandedClaudeCodeView.css";

import { createSvgIconElement } from "../createSvgIconElement";
import type { ClaudeCodeStatusPayload } from "./claudeCodeStatusTypes";
import { CLAUDE_CODE_TERMINAL_ICON_PATH } from "./claudeCodeTerminalIconPath";
import { describeClaudeCodeStatus } from "./describeClaudeCodeStatus";

const CLAUDE_CODE_PRODUCT_NAME = "Claude Code";

export interface ExpandedClaudeCodeView {
  expandedViewElement: HTMLElement;
  showClaudeCodeStatus(claudeCodeStatus: ClaudeCodeStatusPayload): void;
}

// The open pill: which project, what the session is doing, and how many others are active.
export function createExpandedClaudeCodeView(): ExpandedClaudeCodeView {
  const expandedViewElement = document.createElement("div");
  expandedViewElement.className = "expanded-claude-code-view";

  const headerElement = document.createElement("div");
  headerElement.className = "expanded-claude-code-header";
  const terminalIconElement = createSvgIconElement(CLAUDE_CODE_TERMINAL_ICON_PATH);
  terminalIconElement.classList.add("claude-code-terminal-icon", "expanded-claude-code-icon");
  const headerTextElement = document.createElement("div");
  headerTextElement.className = "expanded-claude-code-header-text";
  const projectNameElement = document.createElement("div");
  projectNameElement.className = "expanded-claude-code-project";
  const productNameElement = document.createElement("div");
  productNameElement.className = "expanded-claude-code-product";
  productNameElement.textContent = CLAUDE_CODE_PRODUCT_NAME;
  headerTextElement.append(projectNameElement, productNameElement);
  headerElement.append(terminalIconElement, headerTextElement);

  const statusElement = document.createElement("div");
  statusElement.className = "expanded-claude-code-status";
  const otherSessionsElement = document.createElement("div");
  otherSessionsElement.className = "expanded-claude-code-other-sessions";
  expandedViewElement.append(headerElement, statusElement, otherSessionsElement);

  return {
    expandedViewElement,
    showClaudeCodeStatus(claudeCodeStatus) {
      projectNameElement.textContent = claudeCodeStatus.projectFolderName || CLAUDE_CODE_PRODUCT_NAME;
      statusElement.textContent = describeClaudeCodeStatus(claudeCodeStatus);
      const otherSessionCount = claudeCodeStatus.otherSessionCount;
      otherSessionsElement.textContent =
        otherSessionCount === 0 ? "" : `+${otherSessionCount} other session${otherSessionCount === 1 ? "" : "s"}`;
    },
  };
}
