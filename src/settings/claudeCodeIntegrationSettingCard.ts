import { invoke } from "@tauri-apps/api/core";

import {
  CHANGE_CLAUDE_CODE_INTEGRATION_SETTING_COMMAND,
  CHANGE_CLAUDE_CODE_PERMISSION_ANSWERING_SETTING_COMMAND,
  READ_CLAUDE_CODE_INTEGRATION_SETTING_COMMAND,
} from "../ipc/ipcChannelNames";
import { createSettingRowElement } from "./settingRowElement";

const CLAUDE_CODE_INTEGRATION_SWITCH_ID = "claude-code-integration-switch";
const CLAUDE_CODE_PERMISSION_ANSWERING_SWITCH_ID = "claude-code-permission-answering-switch";

// Mirrors `ClaudeCodeIntegrationSetting` in src-tauri/src/settings_window/claude_code_integration_commands.rs.
interface ClaudeCodeIntegrationSetting {
  isIntegrationOn: boolean;
  isPermissionAnsweringOn: boolean;
  claudeCodeSettingsFilePath: string;
  addedHooksPreview: string;
}

// The Claude Code switch, the switch for answering its permission requests in the pill, and
// exactly what they add to Claude Code's settings file, so nothing changes there without the
// user having been able to read it first. Like the other switches, they always show Rust's
// answer; a failed change flips back and says why.
export async function fillClaudeCodeIntegrationSettingCard(integrationsCard: HTMLElement): Promise<void> {
  const currentSetting = await invoke<ClaudeCodeIntegrationSetting>(READ_CLAUDE_CODE_INTEGRATION_SETTING_COMMAND);
  const integrationSwitch = createSwitchElement(CLAUDE_CODE_INTEGRATION_SWITCH_ID);
  const permissionAnsweringSwitch = createSwitchElement(CLAUDE_CODE_PERMISSION_ANSWERING_SWITCH_ID);

  const addedHooksDetails = document.createElement("details");
  addedHooksDetails.className = "setting-details";
  const addedHooksSummary = document.createElement("summary");
  addedHooksSummary.textContent = `What switching on adds to ${currentSetting.claudeCodeSettingsFilePath}`;
  const addedHooksPreview = document.createElement("pre");
  addedHooksPreview.className = "setting-code-preview";
  const addedHooksNote = document.createElement("p");
  addedHooksNote.className = "setting-row-description";
  addedHooksNote.textContent =
    "A copy of the file is kept first. New Claude Code sessions pick the change up; switching off removes only these entries. Nothing leaves this PC.";
  addedHooksDetails.append(addedHooksSummary, addedHooksPreview, addedHooksNote);

  const changeErrorElement = document.createElement("p");
  changeErrorElement.className = "setting-error";
  changeErrorElement.hidden = true;

  const showSetting = (shownSetting: ClaudeCodeIntegrationSetting): void => {
    integrationSwitch.checked = shownSetting.isIntegrationOn;
    permissionAnsweringSwitch.checked = shownSetting.isPermissionAnsweringOn;
    // Only means something while the integration is on.
    permissionAnsweringSwitch.disabled = !shownSetting.isIntegrationOn;
    addedHooksPreview.textContent = shownSetting.addedHooksPreview;
  };
  showSetting(currentSetting);

  const sendSwitchChange = (changedSwitch: HTMLInputElement, changeCommand: string, changeArguments: Record<string, boolean>): void => {
    integrationSwitch.disabled = true;
    permissionAnsweringSwitch.disabled = true;
    changeErrorElement.hidden = true;
    invoke<ClaudeCodeIntegrationSetting>(changeCommand, changeArguments)
      .then(showSetting)
      .catch((changeError: unknown) => {
        console.error("Crest could not change the Claude Code integration:", changeError);
        changedSwitch.checked = !changedSwitch.checked;
        changeErrorElement.textContent = String(changeError);
        changeErrorElement.hidden = false;
      })
      .finally(() => {
        integrationSwitch.disabled = false;
        permissionAnsweringSwitch.disabled = !integrationSwitch.checked;
      });
  };
  integrationSwitch.addEventListener("change", () =>
    sendSwitchChange(integrationSwitch, CHANGE_CLAUDE_CODE_INTEGRATION_SETTING_COMMAND, {
      shouldIntegrationBeOn: integrationSwitch.checked,
    }),
  );
  permissionAnsweringSwitch.addEventListener("change", () =>
    sendSwitchChange(permissionAnsweringSwitch, CHANGE_CLAUDE_CODE_PERMISSION_ANSWERING_SETTING_COMMAND, {
      shouldPermissionAnsweringBeOn: permissionAnsweringSwitch.checked,
    }),
  );

  integrationsCard.append(
    createSettingRowElement(
      "Claude Code",
      "Shows what a Claude Code session is doing: working, the tool it runs, done, waiting for you, or that it needs your OK.",
      integrationSwitch,
    ),
    createSettingRowElement(
      "Answer permission requests in the pill",
      "Allow or Deny what Claude Code asks for, right in the pill. Claude Code's own question stays open; the first answer wins. Off: the pill only shows the status.",
      permissionAnsweringSwitch,
    ),
    addedHooksDetails,
    changeErrorElement,
  );
}

function createSwitchElement(switchId: string): HTMLInputElement {
  const switchElement = document.createElement("input");
  switchElement.type = "checkbox";
  switchElement.id = switchId;
  switchElement.className = "setting-switch";
  switchElement.setAttribute("role", "switch");
  return switchElement;
}
