import "./styles/designTokens.css";
import "./styles/pillShell.css";

import { PILL_WINDOW_LOGICAL_HEIGHT, PILL_WINDOW_LOGICAL_WIDTH } from "./frontendConstants";
import { listenForPillPresentation } from "./ipc/listenForPillPresentation";
import { requestPillWindowPlacement } from "./ipc/requestPillWindowPlacement";
import { requestPillWindowReveal } from "./ipc/requestPillWindowReveal";
import { PillContentPresenter } from "./pill/pillContentPresenter";
import { applyPillDimensionCssVariables } from "./pill/pillDimensionCssVariables";
import { PillMorphController } from "./pill/pillMorphController";
import { connectPillPointerInput } from "./pill/pillPointerInput";
import { createPillShellElements } from "./pill/pillShellElements";
import { PillStateMachine } from "./pill/pillStateMachine";

// A requestAnimationFrame callback runs just *before* a paint, so waiting for two of
// them guarantees at least one frame with our content has actually been painted.
function waitUntilNextFrameIsPainted(): Promise<void> {
  return new Promise((resolveAfterPaint) =>
    requestAnimationFrame(() => requestAnimationFrame(() => resolveAfterPaint())),
  );
}

async function startPill(): Promise<void> {
  const pillRootElement = document.querySelector<HTMLElement>("#pill-root");
  if (!pillRootElement) {
    throw new Error("index.html has no #pill-root element");
  }
  applyPillDimensionCssVariables();
  const pillShellElements = createPillShellElements();
  pillRootElement.append(pillShellElements.pillShellElement);

  const pillContentPresenter = new PillContentPresenter(pillShellElements);
  const pillMorphController = new PillMorphController(pillShellElements, (isExpandedContentVisible) =>
    pillContentPresenter.setExpandedContentVisible(isExpandedContentVisible),
  );
  const pillStateMachine = new PillStateMachine((expansionState) =>
    pillMorphController.showExpansionState(expansionState),
  );
  connectPillPointerInput(pillShellElements.pillShellElement, pillStateMachine);

  // Window and interactive area first: a track already playing at startup makes the
  // pill peek, which changes the interactive area and must not be overwritten after.
  await requestPillWindowPlacement(PILL_WINDOW_LOGICAL_WIDTH, PILL_WINDOW_LOGICAL_HEIGHT);
  await pillMorphController.applyCompactInteractiveArea();

  let lastAttentionKey: string | null = null;
  await listenForPillPresentation((pillPresentation) => {
    pillContentPresenter.showPillPresentation(pillPresentation);
    const attentionKey = pillPresentation?.attentionKey ?? null;
    if (attentionKey !== null && attentionKey !== lastAttentionKey) {
      pillStateMachine.handleAttentionRequested();
    }
    lastAttentionKey = attentionKey;
  });

  await waitUntilNextFrameIsPainted();
  await requestPillWindowReveal();
}

startPill().catch((startupError: unknown) => {
  console.error("Crest could not start the pill:", startupError);
});
