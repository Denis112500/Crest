import {
  CLAUDE_CODE_EXPANDED_HEADER_LOGICAL_HEIGHT,
  CLAUDE_CODE_EXPANDED_LOGICAL_PADDING,
  CLAUDE_CODE_EXPANDED_SECTION_LOGICAL_GAP,
  CLAUDE_CODE_PERMISSION_BUTTON_LOGICAL_HEIGHT,
  CLAUDE_CODE_PERMISSION_DESCRIPTION_LINE_LOGICAL_HEIGHT,
  CLAUDE_CODE_PERMISSION_DESCRIPTION_LOGICAL_PADDING,
} from "../../frontendConstants";

// How tall the permission alert is for a request shown on this many lines: the same sum the
// flex column in claudeCodePermissionAlertView.css produces (padding, header, gap, the request's
// box, gap, the buttons). Calculated rather than measured, like the session list's height.
export function calculateClaudeCodePermissionAlertHeight(descriptionLineCount: number): number {
  const descriptionHeight =
    descriptionLineCount * CLAUDE_CODE_PERMISSION_DESCRIPTION_LINE_LOGICAL_HEIGHT +
    2 * CLAUDE_CODE_PERMISSION_DESCRIPTION_LOGICAL_PADDING;
  return (
    2 * CLAUDE_CODE_EXPANDED_LOGICAL_PADDING +
    CLAUDE_CODE_EXPANDED_HEADER_LOGICAL_HEIGHT +
    CLAUDE_CODE_EXPANDED_SECTION_LOGICAL_GAP +
    descriptionHeight +
    CLAUDE_CODE_EXPANDED_SECTION_LOGICAL_GAP +
    CLAUDE_CODE_PERMISSION_BUTTON_LOGICAL_HEIGHT
  );
}
