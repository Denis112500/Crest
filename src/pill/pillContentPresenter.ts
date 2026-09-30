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

  constructor(private readonly pillShellElements: PillShellElements) {}

  showPillPresentation(pillPresentation: PillPresentation | null): void {
    const activityKind = pillPresentation?.activityKind ?? null;
    if (!this.mountedViewSet || activityKind !== this.mountedActivityKind) {
      this.mountedViewSet?.setExpandedViewVisible(false);
      const activityViewSet =
        (activityKind !== null && createActivityViewSetForKind(activityKind)) || createNothingToShowViewSet();
      this.pillShellElements.compactLayerElement.replaceChildren(activityViewSet.compactViewElement);
      this.pillShellElements.expandedLayerElement.replaceChildren(activityViewSet.expandedViewElement);
      activityViewSet.setExpandedViewVisible(this.isExpandedContentVisible);
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
}
