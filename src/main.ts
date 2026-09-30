import "./styles/designTokens.css";
import "./styles/pillShell.css";

import { renderPillPresentation } from "./activities/activityViewRegistry";
import { PILL_COMPACT_LOGICAL_HEIGHT, PILL_COMPACT_LOGICAL_WIDTH } from "./frontendConstants";
import { listenForPillPresentation } from "./ipc/listenForPillPresentation";
import { requestPillWindowPlacement } from "./ipc/requestPillWindowPlacement";
import { requestPillWindowReveal } from "./ipc/requestPillWindowReveal";
import { createPillShellElement } from "./pill/pillShellElement";

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
  const pillShellElement = createPillShellElement();
  pillRootElement.append(pillShellElement);

  // Content first, so the very first frame the user sees is already the real one.
  await listenForPillPresentation((pillPresentation) =>
    renderPillPresentation(pillShellElement, pillPresentation),
  );
  await requestPillWindowPlacement(PILL_COMPACT_LOGICAL_WIDTH, PILL_COMPACT_LOGICAL_HEIGHT);
  await waitUntilNextFrameIsPainted();
  await requestPillWindowReveal();
}

startPill().catch((startupError: unknown) => {
  console.error("Crest could not start the pill:", startupError);
});
