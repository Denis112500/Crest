import { IDLE_CLOCK_REDRAW_AFTER_WHOLE_MINUTE_MILLISECONDS } from "../../frontendConstants";

const MILLISECONDS_PER_MINUTE = 60_000;
const MILLISECONDS_PER_SECOND = 1000;

export interface IdleClockTicker {
  /** Whether anyone sees the clock; the timer runs only while seen. */
  setClockSeen(isClockSeen: boolean): void;
}

// Hands the time to a drawing function: at once when the clock becomes seen, then on each new
// minute while it stays seen. Nothing runs while the idle tile isn't on screen.
export function createIdleClockTicker(drawClock: (currentTime: Date) => void): IdleClockTicker {
  let isClockSeenNow = false;
  let redrawTimer: number | undefined;

  // Waits for the next whole minute, so the clock flips on time.
  const scheduleRedrawAtNextWholeMinute = (): void => {
    const currentTime = new Date();
    const millisecondsIntoCurrentMinute = currentTime.getSeconds() * MILLISECONDS_PER_SECOND + currentTime.getMilliseconds();
    redrawTimer = window.setTimeout(() => {
      drawClock(new Date());
      scheduleRedrawAtNextWholeMinute();
    }, MILLISECONDS_PER_MINUTE - millisecondsIntoCurrentMinute + IDLE_CLOCK_REDRAW_AFTER_WHOLE_MINUTE_MILLISECONDS);
  };

  return {
    setClockSeen(isClockSeen) {
      if (isClockSeen === isClockSeenNow) {
        return;
      }
      isClockSeenNow = isClockSeen;
      window.clearTimeout(redrawTimer);
      redrawTimer = undefined;
      if (isClockSeen) {
        drawClock(new Date());
        scheduleRedrawAtNextWholeMinute();
      }
    },
  };
}
