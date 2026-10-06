// Mirrors `ClaudeCodeStatusPayload` in
// src-tauri/src/activity_sources/claude_code/claude_code_session_tracker.rs.
export type ClaudeCodeSessionStatus = "working" | "done" | "waitingForInput" | "needsPermission";

export interface ClaudeCodeStatusPayload {
  sessionStatus: ClaudeCodeSessionStatus;
  /** Only while working on a tool, e.g. "Editing notes.md". */
  toolSummary: string | null;
  projectFolderName: string;
  /** Other sessions with something to show ("+1"). */
  otherSessionCount: number;
}
