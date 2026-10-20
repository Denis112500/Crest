import "../styles/pillDashboardLayout.css";

import type { ActivityViewSet } from "../activities/activityViewSet";
import { PILL_DASHBOARD_TILE_DIVIDER_LOGICAL_WIDTH, PILL_VISOR_EXPANDED_LOGICAL_SLOPE } from "../frontendConstants";

export interface PillDashboardSize {
  logicalWidth: number;
  logicalHeight: number;
}

// The open pill's content: a row of tiles, one per shown activity, with a divider between two
// tiles. Each view keeps one tile and the row is only rebuilt when the tiles change, because
// moving an element in the page restarts what runs in it.
export class PillDashboardLayout {
  private readonly dashboardRowElement = document.createElement("div");
  private readonly tileElementByViewElement = new WeakMap<HTMLElement, HTMLElement>();
  private readonly dividerElements: HTMLElement[] = [];

  constructor(expandedLayerElement: HTMLElement) {
    this.dashboardRowElement.className = "pill-dashboard-row";
    expandedLayerElement.append(this.dashboardRowElement);
  }

  /** Shows these views as tiles, in this order; returns the size the open pill needs for them:
      the tiles plus the Visor's slope on each side (the expanded layer pads them in by it). */
  showTiles(tileViewSets: ActivityViewSet[]): PillDashboardSize {
    const rowChildElements = tileViewSets.flatMap((tileViewSet, tileIndex) => {
      const tileElement = this.findTileElement(tileViewSet.expandedViewElement);
      tileElement.style.width = `${tileViewSet.expandedViewLogicalWidth()}px`;
      return tileIndex === 0 ? [tileElement] : [this.findDividerElement(tileIndex - 1), tileElement];
    });
    const isRowUnchanged =
      rowChildElements.length === this.dashboardRowElement.children.length &&
      rowChildElements.every((rowChildElement, childIndex) => this.dashboardRowElement.children[childIndex] === rowChildElement);
    if (!isRowUnchanged) {
      this.dashboardRowElement.replaceChildren(...rowChildElements);
    }
    return {
      logicalWidth:
        tileViewSets.reduce((tilesWidth, tileViewSet) => tilesWidth + tileViewSet.expandedViewLogicalWidth(), 0) +
        Math.max(0, tileViewSets.length - 1) * PILL_DASHBOARD_TILE_DIVIDER_LOGICAL_WIDTH +
        2 * PILL_VISOR_EXPANDED_LOGICAL_SLOPE,
      logicalHeight: Math.max(0, ...tileViewSets.map((tileViewSet) => tileViewSet.expandedViewLogicalHeight())),
    };
  }

  private findTileElement(expandedViewElement: HTMLElement): HTMLElement {
    let tileElement = this.tileElementByViewElement.get(expandedViewElement);
    if (!tileElement) {
      tileElement = document.createElement("div");
      tileElement.className = "pill-dashboard-tile";
      tileElement.append(expandedViewElement);
      this.tileElementByViewElement.set(expandedViewElement, tileElement);
    }
    return tileElement;
  }

  private findDividerElement(dividerIndex: number): HTMLElement {
    let dividerElement = this.dividerElements[dividerIndex];
    if (!dividerElement) {
      dividerElement = document.createElement("div");
      dividerElement.className = "pill-dashboard-tile-divider";
      this.dividerElements[dividerIndex] = dividerElement;
    }
    return dividerElement;
  }
}
