import type { ClaudeCodeStatusPayload } from "../activities/claudeCode/claudeCodeStatusTypes";
import type { NowPlayingPayload } from "../activities/music/nowPlayingTypes";
import type { PillActivity, PillArrangement } from "../activities/pillArrangementTypes";
import { CLAUDE_CODE_ACTIVITY_KIND, MUSIC_ACTIVITY_KIND } from "../ipc/ipcChannelNames";

/** What the pill's light shows; pillLightController.ts turns it into colours. */
export type PillLightKind = "needsPermission" | "working" | "freshDone" | "song" | "songPaused" | "quiet";

// The light rule (notes/design-spec.md), first match wins: Claude Code needing an OK (gold),
// working (Starlight white, even with music playing), a finish not seen yet (green), then
// music's own colour (never a status colour), dimmed while paused, and a dim white for
// everything else (waiting, a finish already seen, nothing to show). Claude Code's attention key
// only grows with news (a finish, a request), so "not seen yet" means: a session is done and
// that key isn't the one last seen.
export function decidePillLightKind(
  pillArrangement: PillArrangement,
  lastSeenClaudeCodeAttentionKey: string | null,
): PillLightKind {
  const placedActivities = [
    pillArrangement.alertActivity,
    pillArrangement.mainActivity,
    pillArrangement.companionActivity,
    ...pillArrangement.dashboardOnlyActivities,
  ].filter((pillActivity): pillActivity is PillActivity => pillActivity !== null);
  const claudeCodeActivity = placedActivities.find((pillActivity) => pillActivity.activityKind === CLAUDE_CODE_ACTIVITY_KIND);
  const musicActivity = placedActivities.find((pillActivity) => pillActivity.activityKind === MUSIC_ACTIVITY_KIND);
  const sessionStatuses = claudeCodeActivity
    ? (claudeCodeActivity.activityPayload as ClaudeCodeStatusPayload).listedSessions.map(
        (sessionRow) => sessionRow.sessionStatus,
      )
    : [];

  if (pillArrangement.alertActivity !== null || sessionStatuses.includes("needsPermission")) {
    return "needsPermission";
  }
  if (sessionStatuses.includes("working")) {
    return "working";
  }
  if (sessionStatuses.includes("done") && claudeCodeActivity?.attentionKey !== lastSeenClaudeCodeAttentionKey) {
    return "freshDone";
  }
  if (musicActivity) {
    const playbackState = (musicActivity.activityPayload as NowPlayingPayload).playbackState;
    // "changing" is the moment between two tracks: still playing as far as the light goes.
    return playbackState === "playing" || playbackState === "changing" ? "song" : "songPaused";
  }
  return "quiet";
}
