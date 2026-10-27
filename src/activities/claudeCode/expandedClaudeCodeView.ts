import "../../styles/expandedClaudeCodeView.css";

import { CLAUDE_CODE_TURN_STEP_LOGICAL_HEIGHT, PILL_EXPANDED_MAX_LOGICAL_HEIGHT } from "../../frontendConstants";
import { createSvgIconElement } from "../createSvgIconElement";
import { calculateClaudeCodeExpandedHeight } from "./calculateClaudeCodeExpandedHeight";
import { createClaudeCodeSessionRowElement } from "./claudeCodeSessionRowElement";
import { CLAUDE_CODE_PRODUCT_NAME } from "./claudeCodeProductName";
import type { ClaudeCodeStatusPayload } from "./claudeCodeStatusTypes";
import { CLAUDE_CODE_TERMINAL_ICON_PATH } from "./claudeCodeTerminalIconPath";

export interface ExpandedClaudeCodeView {
  expandedViewElement: HTMLElement;
  showClaudeCodeStatus(claudeCodeStatus: ClaudeCodeStatusPayload): void;
  /** How tall the view is for what it shows now, the unfolded row's steps included. */
  logicalHeight(): number;
  /** The pill closed: the next open starts with every row folded. Not a size change to report,
      since nothing is seen until the pill opens again and asks for the height. */
  foldSessions(): void;
  /** Called after a click changed the view's height, so the pill can follow. */
  setSizeChangeListener(sizeChangeListener: () => void): void;
}

// The open pill: how many sessions there are, one row per listed session (most urgent first)
// and "+N more" for the rest. A click on a row unfolds it to its latest steps (CC.3), one row at
// a time; the others then drop their summary lines, so the steps get the room.
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

  let shownClaudeCodeStatus: ClaudeCodeStatusPayload | null = null;
  let unfoldedSessionId: string | null = null;
  let expandedLogicalHeight = calculateClaudeCodeExpandedHeight(1, 0, false, 0);
  let sizeChangeListener = (): void => {};

  const showSessions = (claudeCodeStatus: ClaudeCodeStatusPayload): void => {
    const listedSessions = claudeCodeStatus.listedSessions;
    const unfoldedSession = listedSessions.find((claudeCodeSession) => claudeCodeSession.sessionId === unfoldedSessionId);
    // A session that left the list doesn't come back unfolded.
    unfoldedSessionId = unfoldedSession?.sessionId ?? null;
    const isTurnSummaryShown = (sessionId: string): boolean => !unfoldedSession || sessionId === unfoldedSessionId;
    const shownTurnSummaryCount = listedSessions.filter(
      (claudeCodeSession) => claudeCodeSession.turnSummary !== null && isTurnSummaryShown(claudeCodeSession.sessionId),
    ).length;
    const hasUnlistedSessions = claudeCodeStatus.unlistedSessionCount > 0;
    const heightWithoutSteps = calculateClaudeCodeExpandedHeight(listedSessions.length, shownTurnSummaryCount, hasUnlistedSessions, 0);
    const fittingStepLineCount = Math.max(
      0,
      Math.floor((PILL_EXPANDED_MAX_LOGICAL_HEIGHT - heightWithoutSteps) / CLAUDE_CODE_TURN_STEP_LOGICAL_HEIGHT),
    );
    // An empty turn still takes one line, which says so.
    const unfoldedStepLineCount = unfoldedSession
      ? Math.min(Math.max(unfoldedSession.turnSteps.length, 1), fittingStepLineCount)
      : 0;
    expandedLogicalHeight = heightWithoutSteps + unfoldedStepLineCount * CLAUDE_CODE_TURN_STEP_LOGICAL_HEIGHT;

    const sessionCount = listedSessions.length + claudeCodeStatus.unlistedSessionCount;
    sessionCountElement.textContent = `${sessionCount} session${sessionCount === 1 ? "" : "s"}`;
    sessionListElement.replaceChildren(
      ...listedSessions.map((claudeCodeSession) =>
        createClaudeCodeSessionRowElement(
          claudeCodeSession,
          claudeCodeSession === unfoldedSession ? unfoldedStepLineCount : null,
          isTurnSummaryShown(claudeCodeSession.sessionId),
          () => {
            unfoldedSessionId = claudeCodeSession === unfoldedSession ? null : claudeCodeSession.sessionId;
            showSessions(claudeCodeStatus);
            sizeChangeListener();
          },
        ),
      ),
    );
    // Hidden, not just empty: a hidden element adds no gap, which the height calculation relies on.
    unlistedSessionsElement.hidden = !hasUnlistedSessions;
    unlistedSessionsElement.textContent = `+${claudeCodeStatus.unlistedSessionCount} more`;
  };

  return {
    expandedViewElement,
    showClaudeCodeStatus(claudeCodeStatus) {
      shownClaudeCodeStatus = claudeCodeStatus;
      showSessions(claudeCodeStatus);
    },
    logicalHeight: () => expandedLogicalHeight,
    foldSessions() {
      if (unfoldedSessionId !== null && shownClaudeCodeStatus) {
        unfoldedSessionId = null;
        showSessions(shownClaudeCodeStatus);
      }
    },
    setSizeChangeListener(newSizeChangeListener) {
      sizeChangeListener = newSizeChangeListener;
    },
  };
}
