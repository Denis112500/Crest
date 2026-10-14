import type { ClaudeCodeStatusPayload } from "../activities/claudeCode/claudeCodeStatusTypes";
import type { PillActivity, PillArrangement } from "../activities/pillArrangementTypes";
import { CLAUDE_CODE_ACTIVITY_KIND, MUSIC_ACTIVITY_KIND } from "../ipc/ipcChannelNames";

/** What there is to notice in the pill right now, as one comparable text: when it changes, the
    light sweeps once (pillLightController.ts). */
export interface PillLightNews {
  /** A new track: always worth a sweep. */
  musicTrackKey: string | null;
  /** Claude Code's top line (status and tool) and its news counter: changes every few seconds
      while it works, so its sweeps are spaced out. */
  claudeCodeStatusKey: string | null;
}

// The music activity's attention key is its track; Claude Code's top session row says what it is
// doing right now ("Editing notes.md").
export function describePillLightNews(pillArrangement: PillArrangement): PillLightNews {
  const placedActivities = [
    pillArrangement.alertActivity,
    pillArrangement.mainActivity,
    pillArrangement.companionActivity,
    ...pillArrangement.dashboardOnlyActivities,
  ].filter((pillActivity): pillActivity is PillActivity => pillActivity !== null);
  const musicActivity = placedActivities.find((pillActivity) => pillActivity.activityKind === MUSIC_ACTIVITY_KIND);
  const claudeCodeActivity = placedActivities.find((pillActivity) => pillActivity.activityKind === CLAUDE_CODE_ACTIVITY_KIND);
  const topSessionRow = claudeCodeActivity
    ? (claudeCodeActivity.activityPayload as ClaudeCodeStatusPayload).listedSessions[0]
    : undefined;
  return {
    musicTrackKey: musicActivity?.attentionKey ?? null,
    claudeCodeStatusKey: claudeCodeActivity
      ? [claudeCodeActivity.attentionKey, topSessionRow?.sessionStatus, topSessionRow?.toolSummary].join("|")
      : null,
  };
}
