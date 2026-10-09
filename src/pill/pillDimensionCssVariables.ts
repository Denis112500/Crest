// Hands the sizes from frontendConstants.ts to the CSS, so both use the same numbers.

import {
  CLAUDE_CODE_EXPANDED_HEADER_LOGICAL_HEIGHT,
  CLAUDE_CODE_EXPANDED_LOGICAL_PADDING,
  CLAUDE_CODE_EXPANDED_SECTION_LOGICAL_GAP,
  CLAUDE_CODE_SESSION_ROW_LOGICAL_GAP,
  CLAUDE_CODE_SESSION_ROW_LOGICAL_HEIGHT,
  CLAUDE_CODE_UNLISTED_SESSIONS_LOGICAL_HEIGHT,
  PILL_COMPACT_LOGICAL_HEIGHT,
  PILL_COMPACT_LOGICAL_WIDTH,
  PILL_COMPANION_SEGMENT_LOGICAL_WIDTH,
  PILL_CONCEAL_DURATION_MILLISECONDS,
  PILL_EXPANDED_LOGICAL_WIDTH,
  PILL_MORPH_DURATION_MILLISECONDS,
  PILL_NOTCH_SHOULDER_LOGICAL_RADIUS,
} from "../frontendConstants";

// Lets the CSS use the same numbers as the TypeScript (and therefore as Rust).
export function applyPillDimensionCssVariables(): void {
  const rootStyle = document.documentElement.style;
  rootStyle.setProperty("--pill-compact-width", `${PILL_COMPACT_LOGICAL_WIDTH}px`);
  rootStyle.setProperty("--pill-compact-height", `${PILL_COMPACT_LOGICAL_HEIGHT}px`);
  rootStyle.setProperty("--pill-companion-segment-width", `${PILL_COMPANION_SEGMENT_LOGICAL_WIDTH}px`);
  rootStyle.setProperty("--pill-expanded-width", `${PILL_EXPANDED_LOGICAL_WIDTH}px`);
  rootStyle.setProperty("--pill-notch-shoulder-radius", `${PILL_NOTCH_SHOULDER_LOGICAL_RADIUS}px`);
  rootStyle.setProperty("--pill-morph-duration",`${PILL_MORPH_DURATION_MILLISECONDS}ms`);
  rootStyle.setProperty("--pill-conceal-duration", `${PILL_CONCEAL_DURATION_MILLISECONDS}ms`);
  rootStyle.setProperty("--claude-code-expanded-padding", `${CLAUDE_CODE_EXPANDED_LOGICAL_PADDING}px`);
  rootStyle.setProperty("--claude-code-expanded-header-height", `${CLAUDE_CODE_EXPANDED_HEADER_LOGICAL_HEIGHT}px`);
  rootStyle.setProperty("--claude-code-expanded-section-gap", `${CLAUDE_CODE_EXPANDED_SECTION_LOGICAL_GAP}px`);
  rootStyle.setProperty("--claude-code-session-row-height", `${CLAUDE_CODE_SESSION_ROW_LOGICAL_HEIGHT}px`);
  rootStyle.setProperty("--claude-code-session-row-gap", `${CLAUDE_CODE_SESSION_ROW_LOGICAL_GAP}px`);
  rootStyle.setProperty(
    "--claude-code-unlisted-sessions-height",
    `${CLAUDE_CODE_UNLISTED_SESSIONS_LOGICAL_HEIGHT}px`,
  );
}
