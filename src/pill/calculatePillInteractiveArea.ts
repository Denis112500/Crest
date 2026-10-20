import {
  PILL_COMPACT_LOGICAL_HEIGHT,
  PILL_COMPACT_LOGICAL_WIDTH,
  PILL_COMPANION_MARK_LOGICAL_WIDTH,
  PILL_NOTCH_SHOULDER_LOGICAL_WIDTH,
  PILL_UNDERGLOW_LOGICAL_MARGIN,
  PILL_WINDOW_LOGICAL_WIDTH,
} from "../frontendConstants";

export type PillShape =
  | { expansionState: "compact"; isCompanionMarkShown: boolean }
  | { expansionState: "expanded"; expandedLogicalWidth: number; expandedLogicalHeight: number };

export interface PillInteractiveArea {
  logicalLeft: number;
  logicalTop: number;
  logicalWidth: number;
  logicalHeight: number;
  /** The closed pill is drawn but lets every click through; it opens when the pointer rests on
      it (fix phase P.1/P.2, decided 2026-10-18), so a browser tab under it can still be clicked. */
  isClickThrough: boolean;
}

// The window is sized for the widest and tallest open pill, so only the part the pill covers
// right now may take the mouse; everywhere else, clicks go to the app below (one tile open in
// a window made for two would otherwise block about 200 px on each side). The area also clips
// what is drawn, so it includes the shoulders and the underglow around the pill.
export function calculatePillInteractiveArea(pillShape: PillShape): PillInteractiveArea {
  const pillLogicalWidth =
    pillShape.expansionState === "compact"
      ? PILL_COMPACT_LOGICAL_WIDTH + (pillShape.isCompanionMarkShown ? PILL_COMPANION_MARK_LOGICAL_WIDTH : 0)
      : pillShape.expandedLogicalWidth;
  const pillLogicalHeight =
    pillShape.expansionState === "compact" ? PILL_COMPACT_LOGICAL_HEIGHT : pillShape.expandedLogicalHeight;
  const areaLogicalWidth = pillLogicalWidth + 2 * (PILL_NOTCH_SHOULDER_LOGICAL_WIDTH + PILL_UNDERGLOW_LOGICAL_MARGIN);
  return {
    logicalLeft: (PILL_WINDOW_LOGICAL_WIDTH - areaLogicalWidth) / 2,
    logicalTop: 0,
    logicalWidth: areaLogicalWidth,
    logicalHeight: pillLogicalHeight + PILL_UNDERGLOW_LOGICAL_MARGIN,
    isClickThrough: pillShape.expansionState === "compact",
  };
}
