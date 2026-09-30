import type { ActivityViewSet } from "./activityViewSet";

const NOTHING_TO_SHOW_TEXT = "Nothing to show";

// Shown when no source has anything (or the frontend doesn't know the activity's kind).
export function createNothingToShowViewSet(): ActivityViewSet {
  const createNothingToShowElement = (): HTMLElement => {
    const nothingToShowElement = document.createElement("div");
    nothingToShowElement.className = "pill-nothing-to-show";
    nothingToShowElement.textContent = NOTHING_TO_SHOW_TEXT;
    return nothingToShowElement;
  };
  return {
    compactViewElement: createNothingToShowElement(),
    expandedViewElement: createNothingToShowElement(),
    showActivityPayload: () => {},
    setExpandedViewVisible: () => {},
  };
}
