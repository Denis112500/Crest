// The views of an activity kind this page doesn't know (Rust is newer than the page). With nothing
// on at all, the idle tile shows instead (idle/idleViewSet.ts).

import { NOTHING_TO_SHOW_EXPANDED_LOGICAL_HEIGHT, PILL_DASHBOARD_TILE_LOGICAL_WIDTH } from "../frontendConstants";
import type { ActivityViewSet } from "./activityViewSet";

const NOTHING_TO_SHOW_TEXT = "Nothing to show";

export function createNothingToShowViewSet(): ActivityViewSet {
  const createNothingToShowElement = (): HTMLElement => {
    const nothingToShowElement = document.createElement("div");
    nothingToShowElement.className = "pill-nothing-to-show";
    nothingToShowElement.textContent = NOTHING_TO_SHOW_TEXT;
    return nothingToShowElement;
  };
  return {
    compactViewElement: createNothingToShowElement(),
    companionViewElement: createNothingToShowElement(),
    expandedViewElement: createNothingToShowElement(),
    showActivityPayload: () => {},
    expandedViewLogicalWidth: () => PILL_DASHBOARD_TILE_LOGICAL_WIDTH,
    expandedViewLogicalHeight: () => NOTHING_TO_SHOW_EXPANDED_LOGICAL_HEIGHT,
    setExpandedViewVisible: () => {},
    setCompactPlaceOnScreen: () => {},
  };
}
