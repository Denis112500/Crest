export interface PillShellElements {
  /** The black capsule; its size and corner radius animate between compact and expanded. */
  pillShellElement: HTMLElement;
  compactLayerElement: HTMLElement;
  expandedLayerElement: HTMLElement;
}

// Both layers always exist and cross-fade, so the content never has to be rebuilt when
// the pill changes size. Their shapes and transitions come from pillShell.css.
export function createPillShellElements(): PillShellElements {
  const pillShellElement = document.createElement("div");
  pillShellElement.className = "pill-shell";
  const compactLayerElement = document.createElement("div");
  compactLayerElement.className = "pill-compact-layer";
  const expandedLayerElement = document.createElement("div");
  expandedLayerElement.className = "pill-expanded-layer";
  pillShellElement.append(compactLayerElement, expandedLayerElement);
  return { pillShellElement, compactLayerElement, expandedLayerElement };
}
