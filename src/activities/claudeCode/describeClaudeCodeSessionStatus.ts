import type { ClaudeCodeSessionRow } from "./claudeCodeStatusTypes";

// The line both Claude Code views show for a session: what it is doing right now.
export function describeClaudeCodeSessionStatus(claudeCodeSession: ClaudeCodeSessionRow): string {
  switch (claudeCodeSession.sessionStatus) {
    case "working":
      return claudeCodeSession.toolSummary ?? "Working";
    case "done":
      return "Done";
    case "waitingForInput":
      return "Waiting for you";
    case "needsPermission":
      return "Needs your OK";
  }
}
