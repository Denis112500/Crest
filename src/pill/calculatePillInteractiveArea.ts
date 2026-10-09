import {
  PILL_COMPACT_LOGICAL_HEIGHT,
  PILL_COMPACT_LOGICAL_WIDTH,
  PILL_COMPANION_SEGMENT_LOGICAL_WIDTH,
  PILL_NOTCH_SHOULDER_LOGICAL_RADIUS,
  PILL_SPRING_OVERSHOOT_LOGICAL_MARGIN,
  PILL_WINDOW_LOGICAL_WIDTH,
} from "../frontendConstants";

export type PillShape =
  | { expansionState: "compact"; isCompanionSegmentShown: boolean }
  | { expansionState: "expanded"; expandedLogicalHeight: number };

export interface PillInteractiveArea {
  logicalLeft: number;
  logicalTop: number;
  logicalWidth: number;
  logicalHeight: number;
}

// The window is sized for the tallest open pill, so only the part the pill covers right now
// may take the mouse; everywhere else, clicks go to the app below. The area also clips what is
// drawn, so it includes the shoulders, and while open the spring's overshoot room.
export function calculatePillInteractiveArea(pillShape: PillShape): PillInteractiveArea {
  if (pillShape.expansionState === "compact") {
    const compactLogicalWidth =
      PILL_COMPACT_LOGICAL_WIDTH + (pillShape.isCompanionSegmentShown ? PILL_COMPANION_SEGMENT_LOGICAL_WIDTH : 0);
    const compactNotchLogicalWidth = compactLogicalWidth + 2 * PILL_NOTCH_SHOULDER_LOGICAL_RADIUS;
    return {
      logicalLeft: (PILL_WINDOW_LOGICAL_WIDTH - compactNotchLogicalWidth) / 2,
      logicalTop: 0,
      logicalWidth: compactNotchLogicalWidth,
      logicalHeight: PILL_COMPACT_LOGICAL_HEIGHT,
    };
  }
  return {
    logicalLeft: 0,
    logicalTop: 0,
    logicalWidth: PILL_WINDOW_LOGICAL_WIDTH,
    logicalHeight: pillShape.expandedLogicalHeight + PILL_SPRING_OVERSHOOT_LOGICAL_MARGIN,
  };
}
