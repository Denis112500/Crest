import { invoke } from "@tauri-apps/api/core";

import {
  CHANGE_CLAUDE_CODE_INTEGRATION_SETTING_COMMAND,
  READ_CLAUDE_CODE_INTEGRATION_SETTING_COMMAND,
} from "../ipc/ipcChannelNames";
import { createSettingRowElement } from "./settingRowElement";

const CLAUDE_CODE_INTEGRATION_SWITCH_ID = "claude-code-integration-switch";

// Mirrors `ClaudeCodeIntegrationSetting` in src-tauri/src/settings_window/claude_code_integration_commands.rs.
interface ClaudeCodeIntegrationSetting {
  isIntegrationOn: boolean;
  claudeCodeSettingsFilePath: string;
  addedHooksPreview: string;
}

// The Claude Code switch, plus exactly what switching it on adds to Claude Code's settings file,
// so nothing changes there without the user having been able to read it first. Like the other
// switches, it always shows Rust's answer; a failed change flips back and says why.
export async function fillClaudeCodeIntegrationSettingCard(integrationsCard: HTMLElement): Promise<void> {
  const currentSetting = await invoke<ClaudeCodeIntegrationSetting>(READ_CLAUDE_CODE_INTEGRATION_SETTING_COMMAND);
  const integrationSwitch = document.createElement("input");
  integrationSwitch.type = "checkbox";
  integrationSwitch.id = CLAUDE_CODE_INTEGRATION_SWITCH_ID;
  integrationSwitch.className = "setting-switch";
  integrationSwitch.setAttribute("role", "switch");
  integrationSwitch.checked = currentSetting.isIntegrationOn;

  const addedHooksDetails = document.createElement("details");
  addedHooksDetails.className = "setting-details";
  const addedHooksSummary = document.createElement("summary");
  addedHooksSummary.textContent = `What switching on adds to ${currentSetting.claudeCodeSettingsFilePath}`;
  const addedHooksPreview = document.createElement("pre");
  addedHooksPreview.className = "setting-code-preview";
  addedHooksPreview.textContent = currentSetting.addedHooksPreview;
  const addedHooksNote = document.createElement("p");
  addedHooksNote.className = "setting-row-description";
  addedHooksNote.textContent =
    "A copy of the file is kept first. New Claude Code sessions pick the change up; switching off removes only these entries. Nothing leaves this PC.";
  addedHooksDetails.append(addedHooksSummary, addedHooksPreview, addedHooksNote);

  const changeErrorElement = document.createElement("p");
  changeErrorElement.className = "setting-error";
  changeErrorElement.hidden = true;

  integrationSwitch.addEventListener("change", () => {
    integrationSwitch.disabled = true;
    changeErrorElement.hidden = true;
    invoke<ClaudeCodeIntegrationSetting>(CHANGE_CLAUDE_CODE_INTEGRATION_SETTING_COMMAND, {
      shouldIntegrationBeOn: integrationSwitch.checked,
    })
      .then((changedSetting) => {
        integrationSwitch.checked = changedSetting.isIntegrationOn;
      })
      .catch((changeError: unknown) => {
        console.error("Crest could not change the Claude Code integration:", changeError);
        integrationSwitch.checked = !integrationSwitch.checked;
        changeErrorElement.textContent = String(changeError);
        changeErrorElement.hidden = false;
      })
      .finally(() => {
        integrationSwitch.disabled = false;
      });
  });

  integrationsCard.append(
    createSettingRowElement(
      "Claude Code",
      "Shows what a Claude Code session is doing: working, the tool it runs, done, waiting for you.",
      integrationSwitch,
    ),
    addedHooksDetails,
    changeErrorElement,
  );
}
