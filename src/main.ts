import "./styles/designTokens.css";
import "./styles/pillShell.css";

import type { PillPresentation } from "./activities/pillPresentationTypes";
import { PILL_WINDOW_LOGICAL_HEIGHT, PILL_WINDOW_LOGICAL_WIDTH } from "./frontendConstants";
import {
  GET_CURRENT_PILL_PRESENTATION_COMMAND,
  GET_CURRENT_PILL_VISIBILITY_COMMAND,
  PILL_PRESENTATION_CHANGED_EVENT,
  PILL_VISIBILITY_CHANGED_EVENT,
} from "./ipc/ipcChannelNames";
import { listenForRustStateChanges } from "./ipc/listenForRustStateChanges";
import { requestPillWindowPlacement } from "./ipc/requestPillWindowPlacement";
import { PillContentPresenter } from "./pill/pillContentPresenter";
import { applyPillDimensionCssVariables } from "./pill/pillDimensionCssVariables";
import { PillMorphController } from "./pill/pillMorphController";
import { connectPillPointerInput } from "./pill/pillPointerInput";
import { createPillShellElements } from "./pill/pillShellElements";
import { PillStateMachine } from "./pill/pillStateMachine";
import { PillVisibilityController } from "./pill/pillVisibilityController";
import type { PillVisibility } from "./pill/pillVisibilityTypes";

async function startPill(): Promise<void> {
  const pillRootElement = document.querySelector<HTMLElement>("#pill-root");
  if (!pillRootElement) {
    throw new Error("index.html has no #pill-root element");
  }
  applyPillDimensionCssVariables();
  const pillShellElements = createPillShellElements();
  pillRootElement.append(pillShellElements.pillNotchElement);

  const pillContentPresenter = new PillContentPresenter(pillShellElements);
  const pillMorphController = new PillMorphController(pillShellElements, (isExpandedContentVisible) =>
    pillContentPresenter.setExpandedContentVisible(isExpandedContentVisible),
  );
  const pillStateMachine = new PillStateMachine((expansionState) =>
    pillMorphController.showExpansionState(expansionState),
  );
  // Before the visibility controller: its "pointer left" handler relies on the state
  // machine having seen the same event first.
  connectPillPointerInput(pillShellElements.pillShellElement, pillStateMachine);
  const pillVisibilityController = new PillVisibilityController(
    pillShellElements,
    pillStateMachine,
    (isPillOnScreen) => pillContentPresenter.setPillOnScreen(isPillOnScreen),
  );

  // Window and interactive area first: a track already playing at startup makes the
  // pill peek, which changes the interactive area and must not be overwritten after.
  await requestPillWindowPlacement(PILL_WINDOW_LOGICAL_WIDTH, PILL_WINDOW_LOGICAL_HEIGHT);
  await pillMorphController.applyCompactInteractiveArea();

  let lastAttentionKey: string | null = null;
  await listenForRustStateChanges<PillPresentation | null>(
    PILL_PRESENTATION_CHANGED_EVENT,
    GET_CURRENT_PILL_PRESENTATION_COMMAND,
    (pillPresentation) => {
      pillContentPresenter.showPillPresentation(pillPresentation);
      const attentionKey = pillPresentation?.attentionKey ?? null;
      if (attentionKey !== null && attentionKey !== lastAttentionKey) {
        pillStateMachine.handleAttentionRequested();
      }
      lastAttentionKey = attentionKey;
    },
  );
  // Rust decides whether the pill is on screen; the window stays hidden until it says so.
  await listenForRustStateChanges<PillVisibility>(
    PILL_VISIBILITY_CHANGED_EVENT,
    GET_CURRENT_PILL_VISIBILITY_COMMAND,
    (pillVisibility) => pillVisibilityController.showPillVisibility(pillVisibility),
  );
}

startPill().catch((startupError: unknown) => {
  console.error("Crest could not start the pill:", startupError);
});
