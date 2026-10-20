export interface PillShellElements {
  /** The whole notch: the black Visor with its shoulders, the light along its lower edge and
      the underglow below it. It slides up out of view when the pill hides. */
  pillNotchElement: HTMLElement;
  /** The black Visor; its size animates between compact and expanded, and its outline
      (pillShell.css) follows that size. */
  pillShellElement: HTMLElement;
  compactLayerElement: HTMLElement;
  /** Inside the compact layer: the main activity's small view. */
  compactMainSlotElement: HTMLElement;
  /** Inside the compact layer, at the end of the pill: the companion's mark; shown only while
      there is a companion. Clicking it makes that activity the one the pill opens on. */
  companionMarkElement: HTMLElement;
  expandedLayerElement: HTMLElement;
}

// Both layers always exist and cross-fade, so the content never has to be rebuilt when
// the pill changes size. Their shapes and transitions come from pillShell.css. The light and
// the underglow are the notch's own children behind the Visor, because the Visor clips
// everything outside its outline (it has to, to hide the expanded layer while it's small);
// they take the notch's size, so they follow the Visor while it morphs.
export function createPillShellElements(): PillShellElements {
  const pillNotchElement = document.createElement("div");
  pillNotchElement.className = "pill-notch";
  // The blur sits on the outer element and the outline on the inner one: CSS applies a filter
  // before clipping, so a blurred element that clips itself would cut its own glow off.
  const underglowElement = document.createElement("div");
  underglowElement.className = "pill-underglow";
  const underglowShapeElement = document.createElement("div");
  underglowShapeElement.className = "pill-underglow-shape";
  underglowElement.append(underglowShapeElement);
  const lightRingElement = document.createElement("div");
  lightRingElement.className = "pill-light-ring";
  const pillShellElement = document.createElement("div");
  pillShellElement.className = "pill-shell";
  const compactLayerElement = document.createElement("div");
  compactLayerElement.className = "pill-compact-layer";
  const compactMainSlotElement = document.createElement("div");
  compactMainSlotElement.className = "pill-compact-main-slot";
  // A button for screen readers too, now that no word says what it is: the mark inside gives
  // its name (the companion views label themselves).
  const companionMarkElement = document.createElement("div");
  companionMarkElement.className = "pill-companion-mark";
  companionMarkElement.setAttribute("role", "button");
  compactLayerElement.append(compactMainSlotElement, companionMarkElement);
  const expandedLayerElement = document.createElement("div");
  expandedLayerElement.className = "pill-expanded-layer";
  pillShellElement.append(compactLayerElement, expandedLayerElement);
  pillNotchElement.append(underglowElement, lightRingElement, pillShellElement);
  return {
    pillNotchElement,
    pillShellElement,
    compactLayerElement,
    compactMainSlotElement,
    companionMarkElement,
    expandedLayerElement,
  };
}
