import type { ActivityViewSet } from "../activityViewSet";
import type { ClaudeCodeStatusPayload } from "./claudeCodeStatusTypes";
import { createCompactClaudeCodeView } from "./compactClaudeCodeView";
import { createExpandedClaudeCodeView } from "./expandedClaudeCodeView";

// The "claude-code" activity's views, as registered in activityViewRegistry.ts. They have no
// animations, so there is nothing to pause while hidden.
export function createClaudeCodeViewSet(): ActivityViewSet {
  const compactClaudeCodeView = createCompactClaudeCodeView();
  const expandedClaudeCodeView = createExpandedClaudeCodeView();
  return {
    compactViewElement: compactClaudeCodeView.compactViewElement,
    expandedViewElement: expandedClaudeCodeView.expandedViewElement,
    showActivityPayload(activityPayload) {
      // Rust's Claude Code source always sends this shape (claude_code_session_tracker.rs).
      const claudeCodeStatus = activityPayload as ClaudeCodeStatusPayload;
      compactClaudeCodeView.showClaudeCodeStatus(claudeCodeStatus);
      expandedClaudeCodeView.showClaudeCodeStatus(claudeCodeStatus);
    },
    setExpandedViewVisible: () => {},
    setPillOnScreen: () => {},
  };
}
