import type { ClaudeCodeSessionRow } from "./claudeCodeStatusTypes";
import { formatUsageLimitResetTime } from "./formatUsageLimitResetTime";

// The line both Claude Code views show for a session: what it is doing right now, and the
// subagents it runs that the line doesn't already count ("Editing main.rs · 1 subagent").
export function describeClaudeCodeSessionStatus(claudeCodeSession: ClaudeCodeSessionRow): string {
  const statusLine = describeStatusLine(claudeCodeSession);
  const subagentCount = claudeCodeSession.unmentionedSubagentCount;
  if (subagentCount === 0) {
    return statusLine;
  }
  return `${statusLine} · ${subagentCount} ${subagentCount === 1 ? "subagent" : "subagents"}`;
}

function describeStatusLine(claudeCodeSession: ClaudeCodeSessionRow): string {
  switch (claudeCodeSession.sessionStatus) {
    case "working":
      return claudeCodeSession.toolSummary ?? "Working";
    case "done":
      return "Done";
    case "waitingForInput":
      return "Waiting for you";
    case "needsPermission":
      return "Needs your OK";
    case "usageLimited":
      return claudeCodeSession.usageLimitResetsAt === null
        ? "Usage limit reached"
        : `Usage limit · resets ${formatUsageLimitResetTime(claudeCodeSession.usageLimitResetsAt)}`;
  }
}
