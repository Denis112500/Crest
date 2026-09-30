// The black capsule every activity is drawn inside. Its shape comes from pillShell.css.
export function createPillShellElement(): HTMLElement {
  const pillShellElement = document.createElement("div");
  pillShellElement.className = "pill-shell";
  return pillShellElement;
}
