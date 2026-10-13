// Where an activity's small view is on screen right now: as the main activity, as the companion's
// mark at the end of the pill, or nowhere (not placed small, or the whole pill is hidden).
export type CompactPlaceOnScreen = "main" | "companion" | null;

// Everything one activity kind draws in the pill. A view set is built once per kind and
// then updated in place, so running animations (bars, progress) don't restart on updates.
export interface ActivityViewSet {
  compactViewElement: HTMLElement;
  /** The companion's mark at the end of the closed pill: no word, so it names itself in its
      tooltip and accessible name. */
  companionViewElement: HTMLElement;
  expandedViewElement: HTMLElement;
  showActivityPayload(activityPayload: unknown): void;
  /** How tall the open pill should be for what this activity shows now (logical pixels). */
  expandedViewLogicalHeight(): number;
  /** Lets views pause animations nobody can see, like the progress bar while compact. */
  setExpandedViewVisible(isExpandedViewVisible: boolean): void;
  /** Lets only the small view that's seen animate: none while hidden (during a game) or while
      the activity isn't placed small at all. */
  setCompactPlaceOnScreen(compactPlaceOnScreen: CompactPlaceOnScreen): void;
}
