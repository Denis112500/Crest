export interface PillShellElements {
  /** The whole notch: the capsule plus the two shoulders joining it to the screen edge.
      It slides up out of view when the pill hides. */
  pillNotchElement: HTMLElement;
  /** The black capsule; its size and corner radius animate between compact and expanded. */
  pillShellElement: HTMLElement;
  compactLayerElement: HTMLElement;
  /** Inside the compact layer: the main activity's small view. */
  compactMainSlotElement: HTMLElement;
  /** Inside the compact layer, right of the main slot: the companion's view; shown only while
      there is a companion. Clicking it makes that activity the one the pill opens on. */
  companionSegmentElement: HTMLElement;
  expandedLayerElement: HTMLElement;
}

// Both layers always exist and cross-fade, so the content never has to be rebuilt when
// the pill changes size. Their shapes and transitions come from pillShell.css. The
// shoulders are drawn by the notch element, because the capsule clips everything outside
// itself (it has to, to hide the expanded layer while it's small).
export function createPillShellElements(): PillShellElements {
  const pillNotchElement = document.createElement("div");
  pillNotchElement.className = "pill-notch";
  const pillShellElement = document.createElement("div");
  pillShellElement.className = "pill-shell";
  const compactLayerElement = document.createElement("div");
  compactLayerElement.className = "pill-compact-layer";
  const compactMainSlotElement = document.createElement("div");
  compactMainSlotElement.className = "pill-compact-main-slot";
  const companionSegmentElement = document.createElement("div");
  companionSegmentElement.className = "pill-companion-segment";
  compactLayerElement.append(compactMainSlotElement, companionSegmentElement);
  const expandedLayerElement = document.createElement("div");
  expandedLayerElement.className = "pill-expanded-layer";
  pillShellElement.append(compactLayerElement, expandedLayerElement);
  pillNotchElement.append(pillShellElement);
  return {
    pillNotchElement,
    pillShellElement,
    compactLayerElement,
    compactMainSlotElement,
    companionSegmentElement,
    expandedLayerElement,
  };
}
