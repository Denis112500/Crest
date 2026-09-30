import { MUSIC_ACTIVITY_KIND } from "../ipc/ipcChannelNames";
import { renderCompactMusicView } from "./music/compactMusicView";
import type { PillPresentation } from "./pillPresentationTypes";

type ActivityViewRenderer = (pillShellElement: HTMLElement, activityPayload: unknown) => void;

// The frontend's plugin point: a new activity source adds one line here for its view.
const ACTIVITY_VIEW_RENDERERS_BY_KIND: Record<string, ActivityViewRenderer> = {
  [MUSIC_ACTIVITY_KIND]: renderCompactMusicView,
};

const NOTHING_TO_SHOW_TEXT = "Nothing to show";

export function renderPillPresentation(
  pillShellElement: HTMLElement,
  pillPresentation: PillPresentation | null,
): void {
  const renderActivityView =
    pillPresentation && ACTIVITY_VIEW_RENDERERS_BY_KIND[pillPresentation.activityKind];
  if (pillPresentation && renderActivityView) {
    renderActivityView(pillShellElement, pillPresentation.activityPayload);
    return;
  }
  const nothingToShowElement = document.createElement("span");
  nothingToShowElement.className = "pill-nothing-to-show";
  nothingToShowElement.textContent = NOTHING_TO_SHOW_TEXT;
  pillShellElement.replaceChildren(nothingToShowElement);
}
