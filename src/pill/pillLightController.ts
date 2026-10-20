import type { PillActivity, PillArrangement } from "../activities/pillArrangementTypes";
import { CLAUDE_CODE_ACTIVITY_KIND } from "../ipc/ipcChannelNames";
import { PILL_LIGHT_STATUS_SWEEP_MIN_GAP_MILLISECONDS } from "../frontendConstants";
import { decidePillLightKind, type PillLightKind } from "./decidePillLightKind";
import { describePillLightNews, type PillLightNews } from "./describePillLightNews";
import { type PillLightColors, PillLightPainter } from "./pillLightPainter";
import { readPillLightPalette } from "./readPillLightPalette";

const LIGHT_KIND_ATTRIBUTE = "data-pill-light";

// Decides the pill's light from what Rust sends (decidePillLightKind.ts) and what the user has
// seen, and hands it to the painter. The notch also carries the light's kind, so the marks can
// follow it (pillLight.css).
export class PillLightController {
  private readonly palette = readPillLightPalette();
  private readonly pillLightPainter: PillLightPainter;
  private latestPillArrangement: PillArrangement | null = null;
  private lastSeenClaudeCodeAttentionKey: string | null = null;
  private lastPillLightNews: PillLightNews | null = null;
  private lastStatusSweepTimeMilliseconds = Number.NEGATIVE_INFINITY;

  constructor(private readonly pillNotchElement: HTMLElement) {
    this.pillLightPainter = new PillLightPainter(pillNotchElement);
  }

  showPillArrangement(pillArrangement: PillArrangement): void {
    this.latestPillArrangement = pillArrangement;
    // First the light itself: a change of colour already sweeps, and a sweep for news below then
    // finds one running and adds nothing.
    this.updateLight();
    this.sweepForNews(describePillLightNews(pillArrangement));
  }

  /** The user opened the pill, or pointed at it while it was open: Claude Code's news so far is
      seen, so a finished session's green gives way. An automatic peek doesn't count. */
  markSeenByUser(): void {
    const claudeCodeActivity = this.latestPillArrangement
      ? findPlacedActivity(this.latestPillArrangement, CLAUDE_CODE_ACTIVITY_KIND)
      : null;
    if (claudeCodeActivity && claudeCodeActivity.attentionKey !== this.lastSeenClaudeCodeAttentionKey) {
      this.lastSeenClaudeCodeAttentionKey = claudeCodeActivity.attentionKey;
      this.updateLight();
    }
  }

  setPillOnScreen(isPillOnScreen: boolean): void {
    this.pillLightPainter.setPillOnScreen(isPillOnScreen);
  }

  // A new track always sweeps; Claude Code's status line changes every few seconds while it
  // works, so its sweeps are kept apart (each one repaints the blurred light for a moment). The
  // first arrangement isn't news: it's what was there when Crest started.
  private sweepForNews(pillLightNews: PillLightNews): void {
    const previousPillLightNews = this.lastPillLightNews;
    this.lastPillLightNews = pillLightNews;
    if (!previousPillLightNews) {
      return;
    }
    const isNewTrack =
      pillLightNews.musicTrackKey !== null && pillLightNews.musicTrackKey !== previousPillLightNews.musicTrackKey;
    const isNewStatusLine =
      pillLightNews.claudeCodeStatusKey !== null &&
      pillLightNews.claudeCodeStatusKey !== previousPillLightNews.claudeCodeStatusKey;
    const now = performance.now();
    const isStatusSweepDue = now - this.lastStatusSweepTimeMilliseconds >= PILL_LIGHT_STATUS_SWEEP_MIN_GAP_MILLISECONDS;
    if (isNewTrack || (isNewStatusLine && isStatusSweepDue)) {
      this.lastStatusSweepTimeMilliseconds = now;
      this.pillLightPainter.sweepShownLight();
    }
  }

  private updateLight(): void {
    if (!this.latestPillArrangement) {
      return;
    }
    const lightKind = decidePillLightKind(this.latestPillArrangement, this.lastSeenClaudeCodeAttentionKey);
    this.pillNotchElement.setAttribute(LIGHT_KIND_ATTRIBUTE, lightKind);
    this.pillLightPainter.showLight(this.resolveLightColors(lightKind));
  }

  private resolveLightColors(lightKind: PillLightKind): PillLightColors {
    const [startColor, middleColor, endColor] = {
      needsPermission: this.palette.goldColors,
      working: this.palette.starlightColors,
      freshDone: this.palette.doneColors,
      song: this.palette.musicColors,
      songPaused: this.palette.musicColors,
      quiet: this.palette.starlightColors,
    }[lightKind];
    const strength =
      lightKind === "songPaused"
        ? this.palette.pausedStrength
        : lightKind === "quiet"
          ? this.palette.waitingStrength
          : this.palette.fullStrength;
    return { startColor, middleColor, endColor, strength };
  }
}

function findPlacedActivity(pillArrangement: PillArrangement, activityKind: string): PillActivity | null {
  const placedActivities = [
    pillArrangement.alertActivity,
    pillArrangement.mainActivity,
    pillArrangement.companionActivity,
    ...pillArrangement.dashboardOnlyActivities,
  ];
  return placedActivities.find((pillActivity) => pillActivity?.activityKind === activityKind) ?? null;
}
