import { CLAUDE_CODE_ACTIVITY_KIND, MUSIC_ACTIVITY_KIND } from "../ipc/ipcChannelNames";
import type { PillActivity } from "./pillArrangementTypes";

// Each activity keeps its own place in the open dashboard, so a tile doesn't jump to the left
// when its activity becomes the main one (clicking the companion's mark did that). Which tiles
// are shown is still decided by importance; this only orders them. A user-chosen order can
// replace this list once more integrations exist (decided 2026-10-14).
const DASHBOARD_TILE_POSITION_ORDER = [MUSIC_ACTIVITY_KIND, CLAUDE_CODE_ACTIVITY_KIND];

/** The same activities, left to right in their tile positions; unknown kinds go last, in the
    order they came. */
export function orderActivitiesByTilePosition(dashboardActivities: PillActivity[]): PillActivity[] {
  const findTilePosition = (pillActivity: PillActivity): number => {
    const tilePosition = DASHBOARD_TILE_POSITION_ORDER.indexOf(pillActivity.activityKind);
    return tilePosition === -1 ? DASHBOARD_TILE_POSITION_ORDER.length : tilePosition;
  };
  // Array sort is stable, so activities without a fixed place keep their incoming order.
  return [...dashboardActivities].sort(
    (firstActivity, secondActivity) => findTilePosition(firstActivity) - findTilePosition(secondActivity),
  );
}
