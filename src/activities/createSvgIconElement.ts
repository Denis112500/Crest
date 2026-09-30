const SVG_NAMESPACE = "http://www.w3.org/2000/svg";
const ICON_VIEW_BOX = "0 0 24 24";

// Builds an inline SVG icon from one path drawn on a 24×24 grid. The icon takes the
// text color (`currentColor`) and its size from CSS.
export function createSvgIconElement(iconPathData: string): SVGSVGElement {
  const iconElement = document.createElementNS(SVG_NAMESPACE, "svg");
  iconElement.setAttribute("viewBox", ICON_VIEW_BOX);
  iconElement.setAttribute("aria-hidden", "true");
  iconElement.classList.add("svg-icon");
  const iconPathElement = document.createElementNS(SVG_NAMESPACE, "path");
  iconPathElement.setAttribute("d", iconPathData);
  iconPathElement.setAttribute("fill", "currentColor");
  iconElement.append(iconPathElement);
  return iconElement;
}
