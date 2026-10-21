// Mirrors `PillVisibility` in src-tauri/src/activity_core/pill_visibility_controller.rs.
export interface PillVisibility {
  isPillVisible: boolean;
  // A game or fullscreen video is in front: hide at once instead of fading out over it.
  isFullscreenAppInFront: boolean;
  // On screen because the pointer rested against the top edge: show it open, release it on close.
  isOpenedFromTopEdge: boolean;
}
