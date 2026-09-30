// A requestAnimationFrame callback runs just *before* a paint, so waiting for two of
// them guarantees at least one frame with the current content has actually been painted.
export function waitUntilNextFrameIsPainted(): Promise<void> {
  return new Promise((resolveAfterPaint) =>
    requestAnimationFrame(() => requestAnimationFrame(() => resolveAfterPaint())),
  );
}
