import { listenForRustStateChanges } from "./listenForRustStateChanges";

export interface RustStateFollower {
  /** Starts (listen, then ask once) or stops following; repeating the current choice does nothing. */
  setFollowing(isFollowing: boolean): void;
}

// For Rust state the page needs only while something shows it (the system volume while the open
// music tile is seen): no listener exists the rest of the time. A view that is rebuilt (an
// activity leaving and coming back) can't leave a listener behind, since being hidden stops it.
export function createRustStateFollower<RustState>(
  stateChangedEventName: string,
  getCurrentStateCommandName: string,
  onRustState: (rustState: RustState) => void,
): RustStateFollower {
  let isFollowingNow = false;
  let stopListening: (() => void) | null = null;
  // Changes each time following starts or stops, so a late answer from an earlier start is dropped.
  let followingGeneration = 0;

  return {
    setFollowing(isFollowing) {
      if (isFollowing === isFollowingNow) {
        return;
      }
      isFollowingNow = isFollowing;
      followingGeneration += 1;
      stopListening?.();
      stopListening = null;
      if (!isFollowing) {
        return;
      }
      const thisFollowingGeneration = followingGeneration;
      listenForRustStateChanges<RustState>(stateChangedEventName, getCurrentStateCommandName, (rustState) => {
        if (thisFollowingGeneration === followingGeneration) {
          onRustState(rustState);
        }
      })
        .then((stopListeningToThisStart) => {
          if (thisFollowingGeneration === followingGeneration) {
            stopListening = stopListeningToThisStart;
          } else {
            stopListeningToThisStart();
          }
        })
        .catch((followError: unknown) => {
          console.error(`Crest could not follow "${stateChangedEventName}":`, followError);
        });
    },
  };
}
