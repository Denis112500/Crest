// One row of a settings card, like in Windows' own settings: title and a short
// explanation on the left, the control on the right.
export function createSettingRowElement(
  settingTitle: string,
  settingDescription: string,
  settingControlElement: HTMLElement,
): HTMLElement {
  const settingRowElement = document.createElement("div");
  settingRowElement.className = "setting-row";
  const settingTextElement = document.createElement("div");
  settingTextElement.className = "setting-row-text";
  const settingTitleElement = document.createElement("label");
  settingTitleElement.className = "setting-row-title";
  settingTitleElement.textContent = settingTitle;
  // Clicking the title works the control, as with a native label.
  if (settingControlElement.id) {
    settingTitleElement.htmlFor = settingControlElement.id;
  }
  const settingDescriptionElement = document.createElement("p");
  settingDescriptionElement.className = "setting-row-description";
  settingDescriptionElement.textContent = settingDescription;
  settingTextElement.append(settingTitleElement, settingDescriptionElement);
  settingRowElement.append(settingTextElement, settingControlElement);
  return settingRowElement;
}
