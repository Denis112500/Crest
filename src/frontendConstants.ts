// Pill dimensions in CSS (logical) pixels: the single source of truth. They become CSS
// variables (pillDimensionCssVariables.ts) and are sent to Rust for the window size and
// the interactive area, so the drawn pill and the window can never disagree.
export const PILL_COMPACT_LOGICAL_WIDTH = 220;
export const PILL_COMPACT_LOGICAL_HEIGHT = 36;
// The compact pill grows by this much while a companion sits next to the main activity
// (`[ music | >_ Working ]`): room for its longest word, "Needs OK" (57 px measured), with the
// icon and the rounded end; 220 + 108 still fits inside the open pill's width.
export const PILL_COMPANION_SEGMENT_LOGICAL_WIDTH = 108;
export const PILL_EXPANDED_LOGICAL_WIDTH = 380;
// The open pill is as tall as the shown activity's content (each view set reports its own
// height), up to this much; the window is sized for it.
export const PILL_EXPANDED_MAX_LOGICAL_HEIGHT = 200;
export const MUSIC_EXPANDED_LOGICAL_HEIGHT = 176;
export const NOTHING_TO_SHOW_EXPANDED_LOGICAL_HEIGHT = 96;

// The open Claude Code view: a header, one row per listed session (at most 3, decided in Rust's
// CLAUDE_CODE_LISTED_SESSION_MAX_COUNT), then "+N more". Its height is calculated from these
// (calculateClaudeCodeExpandedHeight.ts), so the CSS gets the same numbers; 3 rows + "+N more"
// = 180, inside PILL_EXPANDED_MAX_LOGICAL_HEIGHT.
export const CLAUDE_CODE_EXPANDED_LOGICAL_PADDING = 16;
export const CLAUDE_CODE_EXPANDED_HEADER_LOGICAL_HEIGHT = 28;
export const CLAUDE_CODE_EXPANDED_SECTION_LOGICAL_GAP = 12;
export const CLAUDE_CODE_SESSION_ROW_LOGICAL_HEIGHT = 24;
export const CLAUDE_CODE_SESSION_ROW_LOGICAL_GAP = 4;
export const CLAUDE_CODE_UNLISTED_SESSIONS_LOGICAL_HEIGHT = 16;

// The spring overshoots its target by about 3%; this much spare room around the expanded
// pill keeps the overshoot from being cut off by the window edge.
export const PILL_SPRING_OVERSHOOT_LOGICAL_MARGIN = 8;

// The curved "shoulders" that join the notch to the screen edge stick out this far on
// each side of the pill, so the window and the interactive area must include them.
export const PILL_NOTCH_SHOULDER_LOGICAL_RADIUS = 8;

// The window always has the largest open size (plus overshoot room and shoulders) and never
// moves or resizes; only its interactive area changes (see calculatePillInteractiveArea.ts).
export const PILL_WINDOW_LOGICAL_WIDTH =
  PILL_EXPANDED_LOGICAL_WIDTH + 2 * (PILL_SPRING_OVERSHOOT_LOGICAL_MARGIN + PILL_NOTCH_SHOULDER_LOGICAL_RADIUS);
export const PILL_WINDOW_LOGICAL_HEIGHT = PILL_EXPANDED_MAX_LOGICAL_HEIGHT + PILL_SPRING_OVERSHOOT_LOGICAL_MARGIN;

export const PILL_MORPH_DURATION_MILLISECONDS = 600;
// Fade-and-shrink when the pill hides, and the reverse when it appears.
export const PILL_CONCEAL_DURATION_MILLISECONDS = 260;
// If the browser skips the transitionend event (e.g. nothing actually changed), the
// collapse or hide is finished by a timer this long after the animation should have ended.
export const PILL_MORPH_END_FALLBACK_SLACK_MILLISECONDS = 100;

// The pointer must rest this long before the pill opens, so moving the mouse past it doesn't.
export const PILL_HOVER_EXPAND_DELAY_MILLISECONDS = 150;
export const PILL_COLLAPSE_DELAY_AFTER_POINTER_LEAVES_MILLISECONDS = 350;
export const PILL_ATTENTION_PEEK_DURATION_MILLISECONDS = 4000;

// Endless animations are driven by timers at a low, fixed rate instead of running at the
// monitor's refresh rate: at 239 Hz, CSS-animated bars cost about 35% of a CPU core.
export const PLAYBACK_BARS_FRAMES_PER_SECOND = 15;
// The progress bar moves about 2 px per second; a few redraws per second look continuous.
export const PLAYBACK_PROGRESS_REDRAWS_PER_SECOND = 4;

// Each bar rises and falls with its own period, so they never move in step.
export const PLAYBACK_BAR_BOUNCE_PERIODS_MILLISECONDS = [1800, 1400, 2100, 1600];
// A resting (paused) bar is this fraction of its full height.
export const PLAYBACK_BAR_RESTING_SCALE = 0.3;
