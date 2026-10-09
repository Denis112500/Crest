import type { ActivityViewSet } from "../activityViewSet";
import { calculateClaudeCodeExpandedHeight } from "./calculateClaudeCodeExpandedHeight";
import type { ClaudeCodeStatusPayload } from "./claudeCodeStatusTypes";
import { createCompactClaudeCodeView } from "./compactClaudeCodeView";
import { createCompanionClaudeCodeView } from "./companionClaudeCodeView";
import { createExpandedClaudeCodeView } from "./expandedClaudeCodeView";

// The "claude-code" activity's views, as registered in activityViewRegistry.ts. They have no
// animations, so there is nothing to pause while hidden.
export function createClaudeCodeViewSet(): ActivityViewSet {
  const compactClaudeCodeView = createCompactClaudeCodeView();
  const companionClaudeCodeView = createCompanionClaudeCodeView();
  const expandedClaudeCodeView = createExpandedClaudeCodeView();
  // Replaced by the first payload, which always comes right after the views are mounted.
  let expandedLogicalHeight = calculateClaudeCodeExpandedHeight(1, false);
  return {
    compactViewElement: compactClaudeCodeView.compactViewElement,
    companionViewElement: companionClaudeCodeView.companionViewElement,
    expandedViewElement: expandedClaudeCodeView.expandedViewElement,
    showActivityPayload(activityPayload) {
      // Rust's Claude Code source always sends this shape (claude_code_session_list.rs).
      const claudeCodeStatus = activityPayload as ClaudeCodeStatusPayload;
      compactClaudeCodeView.showClaudeCodeStatus(claudeCodeStatus);
      companionClaudeCodeView.showClaudeCodeStatus(claudeCodeStatus);
      expandedClaudeCodeView.showClaudeCodeStatus(claudeCodeStatus);
      expandedLogicalHeight = calculateClaudeCodeExpandedHeight(
        claudeCodeStatus.listedSessions.length,
        claudeCodeStatus.unlistedSessionCount > 0,
      );
    },
    expandedViewLogicalHeight: () => expandedLogicalHeight,
    setExpandedViewVisible: () => {},
    setCompactPlaceOnScreen: () => {},
  };
}
