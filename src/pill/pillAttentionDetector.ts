import { findLeadingActivity } from "../activities/findLeadingActivity";
import type { PillActivity, PillArrangement } from "../activities/pillArrangementTypes";

// Decides which activity's news makes the pill peek, from one arrangement to the next:
// - the leading one, when another activity leads now or its attention key changed (a new track);
// - else the companion, when its attention key changed since the page last saw that kind
//   ("Claude Code is done" next to the music). A companion appearing for the first time isn't
//   news: Claude Code starting to work next to the music shouldn't open the pill;
// - right after an alert, only the alert's own activity, and only when the answer brought news
//   (a Deny from the pill). Whatever leads again otherwise isn't news: the answered pill closes.
export class PillAttentionDetector {
  private lastLeadingAttentionKey: string | null = null;
  private readonly lastAttentionKeyByKind = new Map<string, string>();
  private lastAlertActivity: PillActivity | null = null;

  findActivityAskingForAttention(pillArrangement: PillArrangement): PillActivity | null {
    const endedAlertActivity = pillArrangement.alertActivity === null ? this.lastAlertActivity : null;
    this.lastAlertActivity = pillArrangement.alertActivity;
    const leadingActivity = findLeadingActivity(pillArrangement);
    const companionActivity = pillArrangement.companionActivity;
    let activityAskingForAttention: PillActivity | null = null;
    if (leadingActivity && leadingActivity.attentionKey !== this.lastLeadingAttentionKey) {
      activityAskingForAttention = leadingActivity;
    } else if (companionActivity) {
      const lastCompanionAttentionKey = this.lastAttentionKeyByKind.get(companionActivity.activityKind);
      if (lastCompanionAttentionKey !== undefined && lastCompanionAttentionKey !== companionActivity.attentionKey) {
        activityAskingForAttention = companionActivity;
      }
    }
    this.lastLeadingAttentionKey = leadingActivity?.attentionKey ?? null;
    for (const placedActivity of [leadingActivity, companionActivity]) {
      if (placedActivity) {
        this.lastAttentionKeyByKind.set(placedActivity.activityKind, placedActivity.attentionKey);
      }
    }
    if (endedAlertActivity) {
      const answeredActivity = [
        pillArrangement.mainActivity,
        pillArrangement.companionActivity,
        ...pillArrangement.dashboardOnlyActivities,
      ].find((placedActivity) => placedActivity?.activityKind === endedAlertActivity.activityKind);
      return answeredActivity && answeredActivity.attentionKey !== endedAlertActivity.attentionKey ? answeredActivity : null;
    }
    return activityAskingForAttention;
  }
}
