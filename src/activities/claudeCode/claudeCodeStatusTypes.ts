// Mirrors `ClaudeCodeStatusPayload` and `ClaudeCodeSessionRow` in
// src-tauri/src/activity_sources/claude_code/claude_code_session_list.rs.
export type ClaudeCodeSessionStatus = "working" | "done" | "waitingForInput" | "needsPermission";

export interface ClaudeCodeSessionRow {
  sessionStatus: ClaudeCodeSessionStatus;
  /** Only while working on a tool, e.g. "Editing notes.md". */
  toolSummary: string | null;
  projectFolderName: string;
}

export interface ClaudeCodeStatusPayload {
  /** Most urgent first, never empty (Rust withdraws the activity instead); the compact pill
      shows the first one. */
  listedSessions: ClaudeCodeSessionRow[];
  /** Sessions that didn't fit in the open pill ("+2 more"). */
  unlistedSessionCount: number;
  /** Sessions after the first that are working or need an OK ("+1" in the compact pill). */
  otherBusySessionCount: number;
}
