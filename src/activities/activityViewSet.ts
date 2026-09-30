// Everything one activity kind draws in the pill. A view set is built once per kind and
// then updated in place, so running animations (bars, progress) don't restart on updates.
export interface ActivityViewSet {
  compactViewElement: HTMLElement;
  expandedViewElement: HTMLElement;
  showActivityPayload(activityPayload: unknown): void;
  /** Lets views pause animations nobody can see, like the progress bar while compact. */
  setExpandedViewVisible(isExpandedViewVisible: boolean): void;
}
