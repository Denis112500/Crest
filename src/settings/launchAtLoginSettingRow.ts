import { invoke } from "@tauri-apps/api/core";

import {
  CHANGE_LAUNCH_AT_LOGIN_SETTING_COMMAND,
  READ_LAUNCH_AT_LOGIN_SETTING_COMMAND,
} from "../ipc/ipcChannelNames";
import { createSettingRowElement } from "./settingRowElement";

const LAUNCH_AT_LOGIN_SWITCH_ID = "launch-at-login-switch";

// The switch always shows what Windows reports: read when the window opens, and after a
// click it shows Rust's answer, so a failed change flips back instead of lying.
export async function createLaunchAtLoginSettingRow(): Promise<HTMLElement> {
  const launchAtLoginSwitch = document.createElement("input");
  launchAtLoginSwitch.type = "checkbox";
  launchAtLoginSwitch.id = LAUNCH_AT_LOGIN_SWITCH_ID;
  launchAtLoginSwitch.className = "setting-switch";
  launchAtLoginSwitch.setAttribute("role", "switch");
  launchAtLoginSwitch.checked = await invoke<boolean>(READ_LAUNCH_AT_LOGIN_SETTING_COMMAND);
  launchAtLoginSwitch.addEventListener("change", () => {
    launchAtLoginSwitch.disabled = true;
    invoke<boolean>(CHANGE_LAUNCH_AT_LOGIN_SETTING_COMMAND, { shouldLaunchAtLogin: launchAtLoginSwitch.checked })
      .then((isLaunchAtLoginEnabled) => {
        launchAtLoginSwitch.checked = isLaunchAtLoginEnabled;
      })
      .catch((changeError: unknown) => {
        console.error("Crest could not change starting with Windows:", changeError);
        launchAtLoginSwitch.checked = !launchAtLoginSwitch.checked;
      })
      .finally(() => {
        launchAtLoginSwitch.disabled = false;
      });
  });
  return createSettingRowElement(
    "Start with Windows",
    "Crest starts in the background when you sign in.",
    launchAtLoginSwitch,
  );
}
