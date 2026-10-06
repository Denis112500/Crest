// Entry point of the pill page: builds the pill, connects its parts, places the window and
// starts listening to Rust.

import "./styles/designTokens.css";
import "./styles/pillShell.css";

import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { findLeadingActivity } from "./activities/findLeadingActivity";
import type { PillArrangement } from "./activities/pillArrangementTypes";
import { PILL_WINDOW_LOGICAL_HEIGHT, PILL_WINDOW_LOGICAL_WIDTH } from "./frontendConstants";
import {
  GET_CURRENT_PILL_ARRANGEMENT_COMMAND,
  GET_CURRENT_PILL_VISIBILITY_COMMAND,
  PILL_ARRANGEMENT_CHANGED_EVENT,
  PILL_DISPLAY_CHANGED_EVENT,
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

  const placePillWindowOnChosenDisplay = async (): Promise<void> => {
    await requestPillWindowPlacement(PILL_WINDOW_LOGICAL_WIDTH, PILL_WINDOW_LOGICAL_HEIGHT);
    await pillMorphController.applyCurrentInteractiveArea();
  };
  // Window and interactive area first: a track already playing at startup makes the
  // pill peek, which changes the interactive area and must not be overwritten after.
  // Every startup step from here on catches its own failure, so one failed call to Rust
  // can't stop the steps after it; above all the last one, without which the pill would
  // never appear until Crest restarts.
  await placePillWindowOnChosenDisplay().catch(reportPlacementFailure);
  // The user picked another monitor in the settings window.
  await listen(PILL_DISPLAY_CHANGED_EVENT, () => {
    placePillWindowOnChosenDisplay().catch(reportPlacementFailure);
  });
  // Windows rescaled the window (a monitor with other scaling): the interactive area is in
  // physical pixels, so it must be sent again.
  await getCurrentWindow().onScaleChanged(() => {
    placePillWindowOnChosenDisplay().catch(reportPlacementFailure);
  });

  let lastAttentionKey: string | null = null;
  await listenForRustStateChanges<PillArrangement>(
    PILL_ARRANGEMENT_CHANGED_EVENT,
    GET_CURRENT_PILL_ARRANGEMENT_COMMAND,
    (pillArrangement) => {
      pillContentPresenter.showPillArrangement(pillArrangement);
      const attentionKey = findLeadingActivity(pillArrangement)?.attentionKey ?? null;
      if (attentionKey !== null && attentionKey !== lastAttentionKey) {
        pillStateMachine.handleAttentionRequested();
      }
      lastAttentionKey = attentionKey;
    },
  ).catch(reportStartupStepFailure("follow the pill's layout"));
  // Rust decides whether the pill is on screen; the window stays hidden until it says so.
  await listenForRustStateChanges<PillVisibility>(
    PILL_VISIBILITY_CHANGED_EVENT,
    GET_CURRENT_PILL_VISIBILITY_COMMAND,
    (pillVisibility) => pillVisibilityController.showPillVisibility(pillVisibility),
  ).catch(reportStartupStepFailure("follow whether the pill is on screen"));
}

function reportPlacementFailure(placementError: unknown): void {
  console.error("Crest could not place the pill on the chosen display:", placementError);
}

function reportStartupStepFailure(failedStepDescription: string): (startupStepError: unknown) => void {
  return (startupStepError) => console.error(`Crest could not ${failedStepDescription}:`, startupStepError);
}

startPill().catch((startupError: unknown) => {
  console.error("Crest could not start the pill:", startupError);
});
