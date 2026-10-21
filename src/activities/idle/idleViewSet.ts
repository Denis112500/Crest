import "../../styles/idleView.css";

import { IDLE_EXPANDED_LOGICAL_HEIGHT, PILL_DASHBOARD_TILE_LOGICAL_WIDTH } from "../../frontendConstants";
import type { ActivityViewSet } from "../activityViewSet";
import { createSystemVolumeControl } from "../music/systemVolumeControl";
import { createIdleClockTicker } from "./idleClockTicker";

// In the user's own language and clock (24 h or 12 h), as Windows is set up.
const IDLE_TIME_FORMAT = new Intl.DateTimeFormat(undefined, { hour: "numeric", minute: "2-digit" });
const IDLE_DATE_FORMAT = new Intl.DateTimeFormat(undefined, { weekday: "long", day: "numeric", month: "long" });

// What the pill shows with nothing on (P.3b, variant B, decided 2026-10-18): brought down from the
// top edge, the time, the date and the system-volume slider; for the moment it closes before it
// slides away, the time alone. Not an activity, so it never sits at the end as a companion.
export function createIdleViewSet(): ActivityViewSet {
  const compactViewElement = document.createElement("div");
  compactViewElement.className = "compact-idle-view";
  const compactTimeElement = document.createElement("span");
  compactViewElement.append(compactTimeElement);

  const expandedViewElement = document.createElement("div");
  expandedViewElement.className = "expanded-idle-view";
  const clockTextElement = document.createElement("div");
  const expandedTimeElement = document.createElement("div");
  expandedTimeElement.className = "idle-clock-time";
  const dateElement = document.createElement("div");
  dateElement.className = "idle-clock-date";
  clockTextElement.append(expandedTimeElement, dateElement);
  const volumeRowElement = document.createElement("div");
  volumeRowElement.className = "idle-volume-row";
  const systemVolumeControl = createSystemVolumeControl(volumeRowElement, [], () => {}, "alwaysOpen");
  volumeRowElement.append(systemVolumeControl.speakerButtonElement, systemVolumeControl.sliderElement);
  expandedViewElement.append(clockTextElement, volumeRowElement);

  const clockTicker = createIdleClockTicker((currentTime) => {
    const timeText = IDLE_TIME_FORMAT.format(currentTime);
    compactTimeElement.textContent = timeText;
    expandedTimeElement.textContent = timeText;
    dateElement.textContent = IDLE_DATE_FORMAT.format(currentTime);
  });
  let isExpandedViewSeen = false;
  let isCompactViewSeen = false;
  const updateClockSeen = (): void => clockTicker.setClockSeen(isExpandedViewSeen || isCompactViewSeen);

  return {
    compactViewElement,
    companionViewElement: document.createElement("div"),
    expandedViewElement,
    showActivityPayload: () => {},
    expandedViewLogicalWidth: () => PILL_DASHBOARD_TILE_LOGICAL_WIDTH,
    expandedViewLogicalHeight: () => IDLE_EXPANDED_LOGICAL_HEIGHT,
    setExpandedViewVisible(isExpandedViewVisible) {
      isExpandedViewSeen = isExpandedViewVisible;
      systemVolumeControl.setSeen(isExpandedViewVisible);
      updateClockSeen();
    },
    setCompactPlaceOnScreen(compactPlaceOnScreen) {
      isCompactViewSeen = compactPlaceOnScreen === "main";
      updateClockSeen();
    },
  };
}
