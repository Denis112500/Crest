// Pill dimensions in CSS (logical) pixels: the single source of truth. They become CSS
// variables (pillDimensionCssVariables.ts) and are sent to Rust for the window size and
// the interactive area, so the drawn pill and the window can never disagree.
export const PILL_COMPACT_LOGICAL_WIDTH = 220;
export const PILL_COMPACT_LOGICAL_HEIGHT = 36;
export const PILL_EXPANDED_LOGICAL_WIDTH = 380;
export const PILL_EXPANDED_LOGICAL_HEIGHT = 176;

// The spring overshoots its target by about 3%; this much spare room around the expanded
// pill keeps the overshoot from being cut off by the window edge.
export const PILL_SPRING_OVERSHOOT_LOGICAL_MARGIN = 8;

// The window always has the expanded size (plus overshoot room) and never moves or
// resizes; only its interactive area changes (see PillMorphController).
export const PILL_WINDOW_LOGICAL_WIDTH = PILL_EXPANDED_LOGICAL_WIDTH + 2 * PILL_SPRING_OVERSHOOT_LOGICAL_MARGIN;
export const PILL_WINDOW_LOGICAL_HEIGHT = PILL_EXPANDED_LOGICAL_HEIGHT + PILL_SPRING_OVERSHOOT_LOGICAL_MARGIN;

export const PILL_MORPH_DURATION_MILLISECONDS = 600;
// If the browser skips the transitionend event (e.g. nothing actually changed), the
// collapse is finished by a timer this long after the animation should have ended.
export const PILL_MORPH_END_FALLBACK_SLACK_MILLISECONDS = 100;

// The pointer must rest this long before the pill opens, so moving the mouse past it doesn't.
export const PILL_HOVER_EXPAND_DELAY_MILLISECONDS = 150;
export const PILL_COLLAPSE_DELAY_AFTER_POINTER_LEAVES_MILLISECONDS = 350;
export const PILL_ATTENTION_PEEK_DURATION_MILLISECONDS = 4000;
