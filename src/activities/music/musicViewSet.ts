import { MUSIC_EXPANDED_LOGICAL_HEIGHT } from "../../frontendConstants";
import type { ActivityViewSet } from "../activityViewSet";
import { createCompactMusicView } from "./compactMusicView";
import { createCompanionMusicView } from "./companionMusicView";
import { createExpandedMusicView } from "./expandedMusicView";
import type { NowPlayingPayload } from "./nowPlayingTypes";

// The "music" activity's views, as registered in activityViewRegistry.ts.
export function createMusicViewSet(): ActivityViewSet {
  const compactMusicView = createCompactMusicView();
  const companionMusicView = createCompanionMusicView();
  const expandedMusicView = createExpandedMusicView();
  return {
    compactViewElement: compactMusicView.compactViewElement,
    companionViewElement: companionMusicView.companionViewElement,
    expandedViewElement: expandedMusicView.expandedViewElement,
    showActivityPayload(activityPayload) {
      // Rust's music source always sends this shape (media_session_snapshot.rs).
      const nowPlaying = activityPayload as NowPlayingPayload;
      compactMusicView.showNowPlaying(nowPlaying);
      companionMusicView.showNowPlaying(nowPlaying);
      expandedMusicView.showNowPlaying(nowPlaying);
    },
    expandedViewLogicalHeight: () => MUSIC_EXPANDED_LOGICAL_HEIGHT,
    setExpandedViewVisible(isExpandedViewVisible) {
      expandedMusicView.setProgressAnimationActive(isExpandedViewVisible);
    },
    setCompactPlaceOnScreen(compactPlaceOnScreen) {
      compactMusicView.setPillOnScreen(compactPlaceOnScreen === "main");
      companionMusicView.setCompanionOnScreen(compactPlaceOnScreen === "companion");
    },
  };
}
