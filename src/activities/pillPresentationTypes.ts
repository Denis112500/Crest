// Mirrors `PillPresentation` in src-tauri/src/activity_core/activity_arbiter.rs
// (and the flattened `ActivityUpdate` fields). Rust sends `null` when there's nothing to show.
export interface PillPresentation {
  activityKind: string;
  displayPriority: number;
  isOngoing: boolean;
  attentionKey: string;
  /** Shape depends on `activityKind`; each activity's views know their own payload type. */
  activityPayload: unknown;
}
