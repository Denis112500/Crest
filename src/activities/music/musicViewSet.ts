import { MUSIC_EXPANDED_LOGICAL_HEIGHT, PILL_DASHBOARD_TILE_LOGICAL_WIDTH } from "../../frontendConstants";
import type { ActivityViewSet } from "../activityViewSet";
import { createCompactMusicView } from "./compactMusicView";
import { createCompanionMusicView } from "./companionMusicView";
import { createExpandedMusicView } from "./expandedMusicView";
import type { NowPlayingPayload } from "./nowPlayingTypes";
import { type AlbumArtRingColors, sampleAlbumArtRingColors } from "./sampleAlbumArtRingColors";

// The "music" activity's views, as registered in activityViewRegistry.ts. The cover's colours
// for the rings are read once per cover and shared by all three views; an answer for a cover
// that has been replaced meanwhile is dropped.
export function createMusicViewSet(): ActivityViewSet {
  const compactMusicView = createCompactMusicView();
  const companionMusicView = createCompanionMusicView();
  const expandedMusicView = createExpandedMusicView();
  let sampledAlbumArtDataUrl: string | null = null;
  const showRingColors = (albumArtRingColors: AlbumArtRingColors | null): void => {
    compactMusicView.showRingColors(albumArtRingColors);
    companionMusicView.showRingColors(albumArtRingColors);
    expandedMusicView.showRingColors(albumArtRingColors);
  };
  const followAlbumArtColors = (albumArtDataUrl: string | null): void => {
    if (albumArtDataUrl === sampledAlbumArtDataUrl) {
      return;
    }
    sampledAlbumArtDataUrl = albumArtDataUrl;
    showRingColors(null);
    if (albumArtDataUrl !== null) {
      sampleAlbumArtRingColors(albumArtDataUrl).then((albumArtRingColors) => {
        if (albumArtDataUrl === sampledAlbumArtDataUrl) {
          showRingColors(albumArtRingColors);
        }
      });
    }
  };

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
      followAlbumArtColors(nowPlaying.albumArtDataUrl);
    },
    expandedViewLogicalWidth: () => PILL_DASHBOARD_TILE_LOGICAL_WIDTH,
    expandedViewLogicalHeight: () => MUSIC_EXPANDED_LOGICAL_HEIGHT,
    setExpandedViewVisible(isExpandedViewVisible) {
      expandedMusicView.setExpandedViewVisible(isExpandedViewVisible);
    },
    setCompactPlaceOnScreen(compactPlaceOnScreen) {
      compactMusicView.setPillOnScreen(compactPlaceOnScreen === "main");
      companionMusicView.setCompanionOnScreen(compactPlaceOnScreen === "companion");
    },
  };
}
