import { createActivityViewSetForKind } from "../activities/activityViewRegistry";
import type { ActivityViewSet } from "../activities/activityViewSet";
import { findLeadingActivity } from "../activities/findLeadingActivity";
import { orderActivitiesByTilePosition } from "../activities/orderActivitiesByTilePosition";
import { createNothingToShowViewSet } from "../activities/nothingToShowViewSet";
import type { PillActivity, PillArrangement } from "../activities/pillArrangementTypes";
import { PILL_DASHBOARD_MAX_TILE_COUNT } from "../frontendConstants";
import { PillDashboardLayout } from "./pillDashboardLayout";
import type { PillContentLayout } from "./pillMorphController";
import type { PillShellElements } from "./pillShellElements";

// Puts activities' views into the pill: the leading activity (the alert, else the main one)
// in the compact main slot, the companion's mark at the end of the pill. The open pill shows the
// alert alone while there is one; else, after a peek, only the tile with news until it closes;
// else the dashboard: the most important activities (main, companion, then the dashboard-only
// ones), each in its fixed tile position (orderActivitiesByTilePosition.ts). Each activity keeps
// one view set, built when it gets a place and updated in place after, so running animations
// (bars, progress) don't restart on updates.
export class PillContentPresenter {
  private readonly viewSetByActivityKind = new Map<string, ActivityViewSet>();
  private readonly nothingToShowViewSet = createNothingToShowViewSet();
  private readonly pillDashboardLayout: PillDashboardLayout;
  private leadingActivityKind: string | null = null;
  private companionActivityKind: string | null = null;
  private alertActivityKind: string | null = null;
  private dashboardActivityKinds: string[] = [];
  /** The activity a peek opened the pill for; `null` = the whole dashboard. */
  private newsTileActivityKind: string | null = null;
  private isExpandedContentVisible = false;
  private isPillOnScreen = false;

  constructor(
    private readonly pillShellElements: PillShellElements,
    private readonly onContentLayoutChange: (pillContentLayout: PillContentLayout) => void,
  ) {
    this.pillDashboardLayout = new PillDashboardLayout(pillShellElements.expandedLayerElement);
  }

  showPillArrangement(pillArrangement: PillArrangement): void {
    const dashboardActivities = orderActivitiesByTilePosition(
      [pillArrangement.mainActivity, pillArrangement.companionActivity, ...pillArrangement.dashboardOnlyActivities]
        .filter(isPlaced)
        .slice(0, PILL_DASHBOARD_MAX_TILE_COUNT),
    );
    const placedActivities = [pillArrangement.alertActivity, ...dashboardActivities].filter(isPlaced);
    for (const placedActivity of placedActivities) {
      let activityViewSet = this.viewSetByActivityKind.get(placedActivity.activityKind);
      if (!activityViewSet) {
        activityViewSet = createActivityViewSetForKind(placedActivity.activityKind) ?? createNothingToShowViewSet();
        this.viewSetByActivityKind.set(placedActivity.activityKind, activityViewSet);
      }
      activityViewSet.showActivityPayload(placedActivity.activityPayload);
    }
    this.leadingActivityKind = findLeadingActivity(pillArrangement)?.activityKind ?? null;
    this.companionActivityKind = pillArrangement.companionActivity?.activityKind ?? null;
    this.alertActivityKind = pillArrangement.alertActivity?.activityKind ?? null;
    this.dashboardActivityKinds = dashboardActivities.map((dashboardActivity) => dashboardActivity.activityKind);
    this.dropViewSetsNoLongerPlaced(new Set(placedActivities.map((placedActivity) => placedActivity.activityKind)));
    this.mountViews();
  }

  get currentCompanionActivityKind(): string | null {
    return this.companionActivityKind;
  }

  /** News: the peek opens the pill on this activity's tile alone, until it closes. While the
      whole dashboard is open, its tile is already there: the pill doesn't shrink under the user. */
  showNewsTile(activityKind: string): void {
    if (this.isExpandedContentVisible && this.newsTileActivityKind === null) {
      return;
    }
    this.newsTileActivityKind = activityKind;
    this.mountViews();
  }

  setExpandedContentVisible(isExpandedContentVisible: boolean): void {
    this.isExpandedContentVisible = isExpandedContentVisible;
    if (!isExpandedContentVisible) {
      this.newsTileActivityKind = null;
    }
    this.mountViews();
  }

  setPillOnScreen(isPillOnScreen: boolean): void {
    this.isPillOnScreen = isPillOnScreen;
    this.mountViews();
  }

  private dropViewSetsNoLongerPlaced(placedActivityKinds: Set<string>): void {
    for (const [activityKind, activityViewSet] of this.viewSetByActivityKind) {
      if (!placedActivityKinds.has(activityKind)) {
        // The outgoing views must stop their timers before they're dropped: closing a player
        // while it plays used to leave the bars ticking 15 times a second, unseen, forever.
        activityViewSet.setExpandedViewVisible(false);
        activityViewSet.setCompactPlaceOnScreen(null);
        this.viewSetByActivityKind.delete(activityKind);
      }
    }
  }

  private findOpenPillViewSets(): ActivityViewSet[] {
    if (this.alertActivityKind !== null) {
      return [this.findViewSet(this.alertActivityKind)];
    }
    if (this.newsTileActivityKind !== null && this.viewSetByActivityKind.has(this.newsTileActivityKind)) {
      return [this.findViewSet(this.newsTileActivityKind)];
    }
    if (this.dashboardActivityKinds.length === 0) {
      return [this.nothingToShowViewSet];
    }
    return this.dashboardActivityKinds.map((activityKind) => this.findViewSet(activityKind));
  }

  // Every view set learns where it is seen, so the ones nobody sees stop their animations.
  // Their switches are idempotent, so applying them again after any change is safe.
  private mountViews(): void {
    const leadingViewSet = this.findViewSet(this.leadingActivityKind);
    const companionViewSet = this.companionActivityKind === null ? null : this.findViewSet(this.companionActivityKind);
    const openPillViewSets = this.findOpenPillViewSets();
    replaceSlotContentIfDifferent(this.pillShellElements.compactMainSlotElement, leadingViewSet.compactViewElement);
    replaceSlotContentIfDifferent(this.pillShellElements.companionMarkElement, companionViewSet?.companionViewElement ?? null);
    const dashboardSize = this.pillDashboardLayout.showTiles(openPillViewSets);
    for (const activityViewSet of [...this.viewSetByActivityKind.values(), this.nothingToShowViewSet]) {
      const compactPlace =
        activityViewSet === leadingViewSet ? "main" : activityViewSet === companionViewSet ? "companion" : null;
      activityViewSet.setCompactPlaceOnScreen(this.isPillOnScreen ? compactPlace : null);
      activityViewSet.setExpandedViewVisible(this.isExpandedContentVisible && openPillViewSets.includes(activityViewSet));
    }
    this.onContentLayoutChange({
      expandedLogicalWidth: dashboardSize.logicalWidth,
      expandedLogicalHeight: dashboardSize.logicalHeight,
      isCompanionMarkShown: companionViewSet !== null,
    });
  }

  private findViewSet(activityKind: string | null): ActivityViewSet {
    return (activityKind !== null && this.viewSetByActivityKind.get(activityKind)) || this.nothingToShowViewSet;
  }
}

function isPlaced(pillActivity: PillActivity | null): pillActivity is PillActivity {
  return pillActivity !== null;
}

function replaceSlotContentIfDifferent(slotElement: HTMLElement, viewElement: HTMLElement | null): void {
  if (slotElement.firstElementChild !== viewElement) {
    slotElement.replaceChildren(...(viewElement ? [viewElement] : []));
  }
}
