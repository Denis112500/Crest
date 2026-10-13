// Pill dimensions in CSS (logical) pixels: the single source of truth. They become CSS
// variables (pillDimensionCssVariables.ts) and are sent to Rust for the window size and
// the interactive area, so the drawn pill and the window can never disagree.
export const PILL_COMPACT_LOGICAL_WIDTH = 220;
export const PILL_COMPACT_LOGICAL_HEIGHT = 36;
// The compact pill grows by this much while a companion sits next to the main activity: only
// its mark (the `>_` or a tiny cover) at the rounded end, no divider and no word (milestone F,
// mockup option B); one pill height, so the end stays as round as the start.
export const PILL_COMPANION_MARK_LOGICAL_WIDTH = 36;
// The open pill is a dashboard: one tile per activity, side by side, with a thin divider
// between them (layout A, decided 2026-10-12). A peek or an alert is a single tile.
export const PILL_DASHBOARD_TILE_LOGICAL_WIDTH = 380;
export const PILL_DASHBOARD_TILE_DIVIDER_LOGICAL_WIDTH = 1;
// One tile per activity source that exists (music, Claude Code). A third source needs a decision
// first: three tiles would be 1142 wide (notes, milestone E).
export const PILL_DASHBOARD_MAX_TILE_COUNT = 2;
export const PILL_DASHBOARD_MAX_LOGICAL_WIDTH =
  PILL_DASHBOARD_MAX_TILE_COUNT * PILL_DASHBOARD_TILE_LOGICAL_WIDTH +
  (PILL_DASHBOARD_MAX_TILE_COUNT - 1) * PILL_DASHBOARD_TILE_DIVIDER_LOGICAL_WIDTH;
// The open pill is as tall as its tallest tile (each view set reports its own height), up to
// this much; the window is sized for it.
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

// The permission alert: the same padding, header and gaps as above, then the request in a box
// with this inner padding (a command wraps and shows up to 3 rows, scrolling past that), then
// the Allow / Deny row. 3 rows = 182, inside PILL_EXPANDED_MAX_LOGICAL_HEIGHT
// (calculateClaudeCodePermissionAlertHeight.ts adds it up).
export const CLAUDE_CODE_PERMISSION_DESCRIPTION_LINE_LOGICAL_HEIGHT = 18;
export const CLAUDE_CODE_PERMISSION_DESCRIPTION_LOGICAL_PADDING = 6;
export const CLAUDE_CODE_PERMISSION_DESCRIPTION_LOGICAL_PADDING_INLINE = 10;
export const CLAUDE_CODE_PERMISSION_COMMAND_MAX_VISIBLE_ROWS = 3;
// The command box always keeps room for its thin scrollbar, so the text's width (and with it
// the row estimate in estimateClaudeCodeCommandRowCount.ts) is the same with or without one.
export const CLAUDE_CODE_PERMISSION_COMMAND_SCROLLBAR_LOGICAL_WIDTH = 6;
// Only if the page can't measure the command font: narrower than any monospace font at its
// size (Cascadia Mono 12 px is about 7), so the row estimate errs high.
export const CLAUDE_CODE_COMMAND_FALLBACK_CHARACTER_LOGICAL_WIDTH = 6;
// At some display scales a box scrolled to the end stops a fraction of a pixel short of it.
export const CLAUDE_CODE_COMMAND_SCROLL_END_LOGICAL_TOLERANCE = 1;
export const CLAUDE_CODE_PERMISSION_BUTTON_LOGICAL_HEIGHT = 32;
// Allow / Deny ignore clicks this long after the alert becomes visible, so a click meant for
// whatever was under the pointer can't land on them (decided 2026-10-10).
export const CLAUDE_CODE_PERMISSION_BUTTON_ARMING_DELAY_MILLISECONDS = 500;

// The Visor's shoulders curve from the screen edge into its sides; they stick out this far on
// each side of the pill, so the window and the interactive area must include them.
export const PILL_NOTCH_SHOULDER_LOGICAL_WIDTH = 16;

// How far the Visor's sides slope inward from top to bottom. Open, the pill is wider than its
// tiles by this much on each side and the tiles sit inside it, so no content runs into the
// sloped sides (the alert's buttons did, 2026-10-14).
export const PILL_VISOR_COMPACT_LOGICAL_SLOPE = 8;
export const PILL_VISOR_EXPANDED_LOGICAL_SLOPE = 16;

// The underglow (the pill's light falling onto the screen) spreads this far beside and below
// the pill. Windows only draws inside the interactive area (SetWindowRgn), so the area must
// include it; kept small, because Crest also takes the clicks there (decided 2026-10-14).
// designTokens.css keeps the glow's offset and blur inside this margin.
export const PILL_UNDERGLOW_LOGICAL_MARGIN = 12;

// The window always has the largest open size (the widest dashboard, plus the Visor's slopes,
// shoulders and the underglow) and never moves or resizes; only its interactive area changes
// (see calculatePillInteractiveArea.ts). The glide doesn't overshoot, so no spare room is needed.
export const PILL_WINDOW_LOGICAL_WIDTH =
  PILL_DASHBOARD_MAX_LOGICAL_WIDTH +
  2 * (PILL_VISOR_EXPANDED_LOGICAL_SLOPE + PILL_NOTCH_SHOULDER_LOGICAL_WIDTH + PILL_UNDERGLOW_LOGICAL_MARGIN);
export const PILL_WINDOW_LOGICAL_HEIGHT = PILL_EXPANDED_MAX_LOGICAL_HEIGHT + PILL_UNDERGLOW_LOGICAL_MARGIN;

export const PILL_MORPH_DURATION_MILLISECONDS = 560;
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
