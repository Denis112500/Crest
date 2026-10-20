export interface LatestValueThrottle<Value> {
  offer(value: Value): void;
  /** Passes on a value still waiting right away (the drag ended). */
  flush(): void;
}

// Passes values on at most once per interval: the first one at once, then only the latest one
// offered meanwhile, so a fast drag sends a few steps instead of every pixel and always ends on
// its last position. Within an interval the value just passed on isn't passed on again (a click
// reports the same position twice: on input and on release). A timer runs only while values
// keep coming.
export function createLatestValueThrottle<Value>(
  passOn: (value: Value) => void,
  minimumIntervalMilliseconds: number,
): LatestValueThrottle<Value> {
  let waitingValue: { value: Value } | null = null;
  let lastPassedOnValue: { value: Value } | null = null;
  let intervalTimer: number | undefined;

  const passOnWaitingValue = (): void => {
    if (waitingValue !== null) {
      lastPassedOnValue = waitingValue;
      waitingValue = null;
      passOn(lastPassedOnValue.value);
    }
  };
  const endInterval = (): void => {
    intervalTimer = undefined;
    if (waitingValue !== null) {
      passOnWaitingValue();
      intervalTimer = window.setTimeout(endInterval, minimumIntervalMilliseconds);
    } else {
      // Quiet for a whole interval: the value may have changed elsewhere since (the volume
      // keys), so the same value offered later is passed on again.
      lastPassedOnValue = null;
    }
  };

  return {
    offer(value) {
      if (lastPassedOnValue !== null && Object.is(value, lastPassedOnValue.value)) {
        // Back where the last one passed on already is: whatever waited is outdated.
        waitingValue = null;
        return;
      }
      waitingValue = { value };
      if (intervalTimer === undefined) {
        passOnWaitingValue();
        intervalTimer = window.setTimeout(endInterval, minimumIntervalMilliseconds);
      }
    },
    flush() {
      window.clearTimeout(intervalTimer);
      passOnWaitingValue();
      intervalTimer = window.setTimeout(endInterval, minimumIntervalMilliseconds);
    },
  };
}
