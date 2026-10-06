import { CLAUDE_CODE_ACTIVITY_KIND, MUSIC_ACTIVITY_KIND } from "../ipc/ipcChannelNames";
import type { ActivityViewSet } from "./activityViewSet";
import { createClaudeCodeViewSet } from "./claudeCode/claudeCodeViewSet";
import { createMusicViewSet } from "./music/musicViewSet";

// The frontend's plugin point: a new activity source adds one line here for its views.
const ACTIVITY_VIEW_SET_FACTORIES_BY_KIND: Record<string, () => ActivityViewSet> = {
  [MUSIC_ACTIVITY_KIND]: createMusicViewSet,
  [CLAUDE_CODE_ACTIVITY_KIND]: createClaudeCodeViewSet,
};

export function createActivityViewSetForKind(activityKind: string): ActivityViewSet | null {
  const createViewSetForKind = ACTIVITY_VIEW_SET_FACTORIES_BY_KIND[activityKind];
  return createViewSetForKind ? createViewSetForKind() : null;
}
