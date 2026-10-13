import type { PillActivity, PillArrangement } from "../activities/pillArrangementTypes";
import { CLAUDE_CODE_ACTIVITY_KIND } from "../ipc/ipcChannelNames";
import { decidePillLightKind, type PillLightKind } from "./decidePillLightKind";
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

  constructor(private readonly pillNotchElement: HTMLElement) {
    this.pillLightPainter = new PillLightPainter(pillNotchElement);
  }

  showPillArrangement(pillArrangement: PillArrangement): void {
    this.latestPillArrangement = pillArrangement;
    this.updateLight();
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
