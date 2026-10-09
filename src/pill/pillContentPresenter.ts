import { createActivityViewSetForKind } from "../activities/activityViewRegistry";
import type { ActivityViewSet } from "../activities/activityViewSet";
import { findLeadingActivity } from "../activities/findLeadingActivity";
import { createNothingToShowViewSet } from "../activities/nothingToShowViewSet";
import type { PillActivity, PillArrangement } from "../activities/pillArrangementTypes";
import type { PillContentLayout } from "./pillMorphController";
import type { PillShellElements } from "./pillShellElements";

// Puts activities' views into the pill: the leading activity (the alert, else the main one)
// in the compact main slot, the companion in the segment next to it, and in the expanded layer
// the leading one too, except while the pill was opened for another activity (its news, or a
// click on the companion), until it closes again. Each placed activity keeps one view set,
// built when it gets its place and updated in place after, so running animations (bars,
// progress) don't restart on updates.
export class PillContentPresenter {
  private readonly viewSetByActivityKind = new Map<string, ActivityViewSet>();
  private readonly nothingToShowViewSet = createNothingToShowViewSet();
  private placedActivityKinds = new Set<string>();
  private leadingActivityKind: string | null = null;
  private companionActivityKind: string | null = null;
  /** The activity the pill was opened for; `null` = the leading one. */
  private expandedActivityKindOverride: string | null = null;
  private isExpandedContentVisible = false;
  private isPillOnScreen = false;

  constructor(
    private readonly pillShellElements: PillShellElements,
    private readonly onContentLayoutChange: (pillContentLayout: PillContentLayout) => void,
  ) {}

  showPillArrangement(pillArrangement: PillArrangement): void {
    const leadingActivity = findLeadingActivity(pillArrangement);
    const placedActivities = [leadingActivity, pillArrangement.companionActivity].filter(
      (placedActivity): placedActivity is PillActivity => placedActivity !== null,
    );
    this.placedActivityKinds = new Set(placedActivities.map((placedActivity) => placedActivity.activityKind));
    for (const placedActivity of placedActivities) {
      let activityViewSet = this.viewSetByActivityKind.get(placedActivity.activityKind);
      if (!activityViewSet) {
        activityViewSet = createActivityViewSetForKind(placedActivity.activityKind) ?? createNothingToShowViewSet();
        this.viewSetByActivityKind.set(placedActivity.activityKind, activityViewSet);
      }
      activityViewSet.showActivityPayload(placedActivity.activityPayload);
    }
    this.leadingActivityKind = leadingActivity?.activityKind ?? null;
    this.companionActivityKind = pillArrangement.companionActivity?.activityKind ?? null;
    this.dropViewSetsNoLongerNeeded();
    this.mountViews();
  }

  get currentCompanionActivityKind(): string | null {
    return this.companionActivityKind;
  }

  /** The pill opens for this activity: its expanded view, until the pill closes. */
  showActivityInExpandedView(activityKind: string): void {
    this.expandedActivityKindOverride = activityKind === this.leadingActivityKind ? null : activityKind;
    this.mountViews();
  }

  setExpandedContentVisible(isExpandedContentVisible: boolean): void {
    this.isExpandedContentVisible = isExpandedContentVisible;
    if (!isExpandedContentVisible) {
      this.expandedActivityKindOverride = null;
      this.dropViewSetsNoLongerNeeded();
    }
    this.mountViews();
  }

  setPillOnScreen(isPillOnScreen: boolean): void {
    this.isPillOnScreen = isPillOnScreen;
    this.mountViews();
  }

  // An activity that lost its place is dropped, except the one the open pill shows: a clicked
  // "Done" next to the music counts as seen and leaves at once, but its list stays readable
  // until the pill closes.
  private dropViewSetsNoLongerNeeded(): void {
    for (const [activityKind, activityViewSet] of this.viewSetByActivityKind) {
      const isShownInOpenPill = this.isExpandedContentVisible && activityKind === this.expandedActivityKindOverride;
      if (!this.placedActivityKinds.has(activityKind) && !isShownInOpenPill) {
        // The outgoing views must stop their timers before they're dropped: closing a player
        // while it plays used to leave the bars ticking 15 times a second, unseen, forever.
        activityViewSet.setExpandedViewVisible(false);
        activityViewSet.setCompactPlaceOnScreen(null);
        this.viewSetByActivityKind.delete(activityKind);
      }
    }
  }

  // Every view set learns where it is seen, so the ones nobody sees stop their animations.
  // Their switches are idempotent, so applying them again after any change is safe.
  private mountViews(): void {
    const leadingViewSet = this.findViewSet(this.leadingActivityKind);
    const companionViewSet = this.companionActivityKind === null ? null : this.findViewSet(this.companionActivityKind);
    const isOverrideStillThere =
      this.expandedActivityKindOverride !== null && this.viewSetByActivityKind.has(this.expandedActivityKindOverride);
    const expandedViewSet = isOverrideStillThere ? this.findViewSet(this.expandedActivityKindOverride) : leadingViewSet;
    replaceSlotContentIfDifferent(this.pillShellElements.compactMainSlotElement, leadingViewSet.compactViewElement);
    replaceSlotContentIfDifferent(this.pillShellElements.companionSegmentElement, companionViewSet?.companionViewElement ?? null);
    replaceSlotContentIfDifferent(this.pillShellElements.expandedLayerElement, expandedViewSet.expandedViewElement);
    for (const activityViewSet of [...this.viewSetByActivityKind.values(), this.nothingToShowViewSet]) {
      const compactPlace =
        activityViewSet === leadingViewSet ? "main" : activityViewSet === companionViewSet ? "companion" : null;
      activityViewSet.setCompactPlaceOnScreen(this.isPillOnScreen ? compactPlace : null);
      activityViewSet.setExpandedViewVisible(this.isExpandedContentVisible && activityViewSet === expandedViewSet);
    }
    this.onContentLayoutChange({
      expandedLogicalHeight: expandedViewSet.expandedViewLogicalHeight(),
      isCompanionSegmentShown: companionViewSet !== null,
    });
  }

  private findViewSet(activityKind: string | null): ActivityViewSet {
    return (activityKind !== null && this.viewSetByActivityKind.get(activityKind)) || this.nothingToShowViewSet;
  }
}

function replaceSlotContentIfDifferent(slotElement: HTMLElement, viewElement: HTMLElement | null): void {
  if (slotElement.firstElementChild !== viewElement) {
    slotElement.replaceChildren(...(viewElement ? [viewElement] : []));
  }
}
