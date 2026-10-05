import { createActivityViewSetForKind } from "../activities/activityViewRegistry";
import type { ActivityViewSet } from "../activities/activityViewSet";
import { createNothingToShowViewSet } from "../activities/nothingToShowViewSet";
import type { PillPresentation } from "../activities/pillPresentationTypes";
import type { PillShellElements } from "./pillShellElements";

// Puts the right activity's views into the pill's two layers. Views are only rebuilt
// when the activity kind changes; otherwise they are updated in place.
export class PillContentPresenter {
  private mountedActivityKind: string | null = null;
  private mountedViewSet: ActivityViewSet | undefined;
  private isExpandedContentVisible = false;
  private isPillOnScreen = false;

  constructor(private readonly pillShellElements: PillShellElements) {}

  showPillPresentation(pillPresentation: PillPresentation | null): void {
    const activityKind = pillPresentation?.activityKind ?? null;
    if (!this.mountedViewSet || activityKind !== this.mountedActivityKind) {
      // The outgoing views must stop their timers before they're dropped: closing a player
      // while it plays used to leave the bars ticking 15 times a second, unseen, forever.
      this.mountedViewSet?.setExpandedViewVisible(false);
      this.mountedViewSet?.setPillOnScreen(false);
      const activityViewSet =
        (activityKind !== null && createActivityViewSetForKind(activityKind)) || createNothingToShowViewSet();
      this.pillShellElements.compactLayerElement.replaceChildren(activityViewSet.compactViewElement);
      this.pillShellElements.expandedLayerElement.replaceChildren(activityViewSet.expandedViewElement);
      activityViewSet.setExpandedViewVisible(this.isExpandedContentVisible);
      activityViewSet.setPillOnScreen(this.isPillOnScreen);
      this.mountedViewSet = activityViewSet;
      this.mountedActivityKind = activityKind;
    }
    if (pillPresentation) {
      this.mountedViewSet.showActivityPayload(pillPresentation.activityPayload);
    }
  }

  setExpandedContentVisible(isExpandedContentVisible: boolean): void {
    this.isExpandedContentVisible = isExpandedContentVisible;
    this.mountedViewSet?.setExpandedViewVisible(isExpandedContentVisible);
  }

  setPillOnScreen(isPillOnScreen: boolean): void {
    this.isPillOnScreen = isPillOnScreen;
    this.mountedViewSet?.setPillOnScreen(isPillOnScreen);
  }
}
