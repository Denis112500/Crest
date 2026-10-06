// The "Show the pill on" monitor list.

import { invoke } from "@tauri-apps/api/core";

import { CHOOSE_PILL_DISPLAY_COMMAND, LIST_PILL_DISPLAY_OPTIONS_COMMAND } from "../ipc/ipcChannelNames";
import { createSettingRowElement } from "./settingRowElement";

// Matches `PillDisplayOption` in src-tauri/src/pill_window/pill_display_choice.rs.
interface PillDisplayOption {
  displayName: string | null;
  label: string;
  isChosen: boolean;
}

const PILL_DISPLAY_SELECT_ID = "pill-display-select";
// A <select> value is always a string; "Main display" (no name) is stored as the empty one.
const MAIN_DISPLAY_OPTION_VALUE = "";

export async function createPillDisplaySettingRow(): Promise<HTMLElement> {
  const pillDisplaySelect = document.createElement("select");
  pillDisplaySelect.id = PILL_DISPLAY_SELECT_ID;
  pillDisplaySelect.className = "setting-select";
  const pillDisplayOptions = await invoke<PillDisplayOption[]>(LIST_PILL_DISPLAY_OPTIONS_COMMAND);
  for (const pillDisplayOption of pillDisplayOptions) {
    const optionElement = document.createElement("option");
    optionElement.value = pillDisplayOption.displayName ?? MAIN_DISPLAY_OPTION_VALUE;
    optionElement.textContent = pillDisplayOption.label;
    optionElement.selected = pillDisplayOption.isChosen;
    pillDisplaySelect.append(optionElement);
  }
  pillDisplaySelect.addEventListener("change", () => {
    const chosenDisplayName = pillDisplaySelect.value === MAIN_DISPLAY_OPTION_VALUE ? null : pillDisplaySelect.value;
    invoke(CHOOSE_PILL_DISPLAY_COMMAND, { displayName: chosenDisplayName }).catch((chooseError: unknown) => {
      console.error("Crest could not move the pill to the chosen display:", chooseError);
    });
  });
  return createSettingRowElement(
    "Show the pill on",
    "The pill hangs from the top center of this display.",
    pillDisplaySelect,
  );
}
