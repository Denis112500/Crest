import {
  CLAUDE_CODE_EXPANDED_HEADER_LOGICAL_HEIGHT,
  CLAUDE_CODE_EXPANDED_LOGICAL_PADDING,
  CLAUDE_CODE_EXPANDED_SECTION_LOGICAL_GAP,
  CLAUDE_CODE_SESSION_ROW_LOGICAL_GAP,
  CLAUDE_CODE_SESSION_ROW_LOGICAL_HEIGHT,
  CLAUDE_CODE_UNLISTED_SESSIONS_LOGICAL_HEIGHT,
} from "../../frontendConstants";

// How tall the open Claude Code view is for this many rows: the same sum the flex column in
// expandedClaudeCodeView.css produces (padding, header, gap, rows with their gaps, and the
// "+N more" line with its gap only when it is shown). Calculated rather than measured, so the
// pill can start growing before the new rows are laid out.
export function calculateClaudeCodeExpandedHeight(listedSessionCount: number, hasUnlistedSessions: boolean): number {
  const sessionListHeight =
    listedSessionCount * CLAUDE_CODE_SESSION_ROW_LOGICAL_HEIGHT +
    Math.max(listedSessionCount - 1, 0) * CLAUDE_CODE_SESSION_ROW_LOGICAL_GAP;
  const unlistedSessionsHeight = hasUnlistedSessions
    ? CLAUDE_CODE_EXPANDED_SECTION_LOGICAL_GAP + CLAUDE_CODE_UNLISTED_SESSIONS_LOGICAL_HEIGHT
    : 0;
  return (
    2 * CLAUDE_CODE_EXPANDED_LOGICAL_PADDING +
    CLAUDE_CODE_EXPANDED_HEADER_LOGICAL_HEIGHT +
    CLAUDE_CODE_EXPANDED_SECTION_LOGICAL_GAP +
    sessionListHeight +
    unlistedSessionsHeight
  );
}
