import {
  PILL_COMPACT_LOGICAL_HEIGHT,
  PILL_COMPACT_LOGICAL_WIDTH,
  PILL_EXPANDED_LOGICAL_HEIGHT,
  PILL_EXPANDED_LOGICAL_WIDTH,
  PILL_MORPH_DURATION_MILLISECONDS,
} from "../frontendConstants";

// Lets the CSS use the same numbers as the TypeScript (and therefore as Rust).
export function applyPillDimensionCssVariables(): void {
  const rootStyle = document.documentElement.style;
  rootStyle.setProperty("--pill-compact-width", `${PILL_COMPACT_LOGICAL_WIDTH}px`);
  rootStyle.setProperty("--pill-compact-height", `${PILL_COMPACT_LOGICAL_HEIGHT}px`);
  rootStyle.setProperty("--pill-expanded-width", `${PILL_EXPANDED_LOGICAL_WIDTH}px`);
  rootStyle.setProperty("--pill-expanded-height", `${PILL_EXPANDED_LOGICAL_HEIGHT}px`);
  rootStyle.setProperty("--pill-morph-duration", `${PILL_MORPH_DURATION_MILLISECONDS}ms`);
}
