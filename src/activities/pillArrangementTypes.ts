// Mirrors `PillArrangement` and `PillActivity` in
// src-tauri/src/activity_core/pill_activity_arrangement.rs (with the flattened `ActivityUpdate`
// fields). Any of the three places may be `null`; all `null` means nothing to show.
export type ActivityPresence = "lingering" | "ongoing" | "alert";

export interface PillActivity {
  activityKind: string;
  displayPriority: number;
  activityPresence: ActivityPresence;
  attentionKey: string;
  /** Shape depends on `activityKind`; each activity's views know their own payload type. */
  activityPayload: unknown;
}

export interface PillArrangement {
  alertActivity: PillActivity | null;
  mainActivity: PillActivity | null;
  companionActivity: PillActivity | null;
  /** Ongoing activities beyond main and companion (the companion shows "+N"). */
  otherOngoingCount: number;
}
