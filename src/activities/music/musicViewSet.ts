import type { ActivityViewSet } from "../activityViewSet";
import { createCompactMusicView } from "./compactMusicView";
import { createExpandedMusicView } from "./expandedMusicView";
import type { NowPlayingPayload } from "./nowPlayingTypes";

// The "music" activity's views, as registered in activityViewRegistry.ts.
export function createMusicViewSet(): ActivityViewSet {
  const compactMusicView = createCompactMusicView();
  const expandedMusicView = createExpandedMusicView();
  return {
    compactViewElement: compactMusicView.compactViewElement,
    expandedViewElement: expandedMusicView.expandedViewElement,
    showActivityPayload(activityPayload) {
      // Rust's music source always sends this shape (media_session_snapshot.rs).
      const nowPlaying = activityPayload as NowPlayingPayload;
      compactMusicView.showNowPlaying(nowPlaying);
      expandedMusicView.showNowPlaying(nowPlaying);
    },
    setExpandedViewVisible(isExpandedViewVisible) {
      expandedMusicView.setProgressAnimationActive(isExpandedViewVisible);
    },
  };
}
