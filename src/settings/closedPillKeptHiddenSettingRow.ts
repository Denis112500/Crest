import { invoke } from "@tauri-apps/api/core";

import {
  CHANGE_CLOSED_PILL_KEPT_HIDDEN_SETTING_COMMAND,
  READ_CLOSED_PILL_KEPT_HIDDEN_SETTING_COMMAND,
} from "../ipc/ipcChannelNames";
import { createSettingRowElement } from "./settingRowElement";

const CLOSED_PILL_KEPT_HIDDEN_SWITCH_ID = "closed-pill-kept-hidden-switch";

// The switch shows what Rust saved: a failed save flips it back instead of lying.
export async function createClosedPillKeptHiddenSettingRow(): Promise<HTMLElement> {
  const keptHiddenSwitch = document.createElement("input");
  keptHiddenSwitch.type = "checkbox";
  keptHiddenSwitch.id = CLOSED_PILL_KEPT_HIDDEN_SWITCH_ID;
  keptHiddenSwitch.className = "setting-switch";
  keptHiddenSwitch.setAttribute("role", "switch");
  keptHiddenSwitch.checked = await invoke<boolean>(READ_CLOSED_PILL_KEPT_HIDDEN_SETTING_COMMAND);
  keptHiddenSwitch.addEventListener("change", () => {
    keptHiddenSwitch.disabled = true;
    invoke<boolean>(CHANGE_CLOSED_PILL_KEPT_HIDDEN_SETTING_COMMAND, {
      shouldKeepClosedPillHidden: keptHiddenSwitch.checked,
    })
      .then((isClosedPillKeptHidden) => {
        keptHiddenSwitch.checked = isClosedPillKeptHidden;
      })
      .catch((changeError: unknown) => {
        console.error("Crest could not change keeping the closed pill hidden:", changeError);
        keptHiddenSwitch.checked = !keptHiddenSwitch.checked;
      })
      .finally(() => {
        keptHiddenSwitch.disabled = false;
      });
  });
  return createSettingRowElement(
    "Keep the closed pill hidden",
    "Music and Claude Code no longer keep the pill on screen. Rest the pointer at the top edge to open it; permission requests still come down.",
    keptHiddenSwitch,
  );
}
