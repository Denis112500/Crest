// Mirrors `ClaudeCodeStatusPayload` and `ClaudeCodeSessionRow` in
// src-tauri/src/activity_sources/claude_code/claude_code_session_list.rs, and the permission
// alert in claude_code_permission_queue.rs and claude_code_permission_description.rs.
export type ClaudeCodeSessionStatus = "working" | "done" | "waitingForInput" | "needsPermission";

export interface ClaudeCodeSessionRow {
  sessionStatus: ClaudeCodeSessionStatus;
  /** Only while working on a tool, e.g. "Editing notes.md". */
  toolSummary: string | null;
  projectFolderName: string;
}

/** A command as it will run (lines separated by "\n", hidden characters shown as "�"), or a
    short action like "Edit notes.md". `isCommandCut`: too long to send whole, ends in "…". */
export type ClaudeCodePermissionDescription =
  | { descriptionKind: "command"; commandText: string; isCommandCut: boolean }
  | { descriptionKind: "action"; actionText: string };

export interface ClaudeCodePermissionAlert {
  /** Sent back with the click, so it can only answer this request. */
  requestId: number;
  projectFolderName: string;
  requestDescription: ClaudeCodePermissionDescription;
  /** Requests behind this one, from any session ("+1 waiting"). */
  otherWaitingRequestCount: number;
}

export interface ClaudeCodeStatusPayload {
  /** Most urgent first, never empty (Rust withdraws the activity instead); the compact pill
      shows the first one. */
  listedSessions: ClaudeCodeSessionRow[];
  /** Sessions that didn't fit in the open pill ("+2 more"). */
  unlistedSessionCount: number;
  /** Sessions after the first that are working or need an OK ("+1" in the compact pill). */
  otherBusySessionCount: number;
  /** The permission request waiting for Allow or Deny; while there is one, the activity is
      the pill's alert and the open pill shows this instead of the session list. */
  permissionAlert: ClaudeCodePermissionAlert | null;
}
