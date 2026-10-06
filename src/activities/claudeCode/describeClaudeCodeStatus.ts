import type { ClaudeCodeStatusPayload } from "./claudeCodeStatusTypes";

// The one line both Claude Code views show: what the session is doing right now.
export function describeClaudeCodeStatus(claudeCodeStatus: ClaudeCodeStatusPayload): string {
  switch (claudeCodeStatus.sessionStatus) {
    case "working":
      return claudeCodeStatus.toolSummary ?? "Working";
    case "done":
      return "Done";
    case "waitingForInput":
      return "Waiting for you";
    case "needsPermission":
      return "Needs your OK";
  }
}
