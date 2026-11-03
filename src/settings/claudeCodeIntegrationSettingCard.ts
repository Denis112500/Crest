import { invoke } from "@tauri-apps/api/core";

import {
  CHANGE_CLAUDE_CODE_INTEGRATION_SETTING_COMMAND,
  CHANGE_CLAUDE_CODE_PERMISSION_ANSWERING_SETTING_COMMAND,
  PREVIEW_CLAUDE_CODE_SWITCH_CHANGE_COMMAND,
  READ_CLAUDE_CODE_INTEGRATION_SETTING_COMMAND,
} from "../ipc/ipcChannelNames";
import { fillClaudeCodeSwitchChangePanel } from "./claudeCodeSwitchChangePanel";
import type {
  ClaudeCodeIntegrationSetting,
  ClaudeCodeSwitchChange,
  ClaudeCodeSwitchChangeOutcome,
  ClaudeCodeSwitchChangePreview,
} from "./claudeCodeSwitchChangeTypes";
import { createSettingRowElement } from "./settingRowElement";

const CLAUDE_CODE_INTEGRATION_SWITCH_ID = "claude-code-integration-switch";
const CLAUDE_CODE_PERMISSION_ANSWERING_SWITCH_ID = "claude-code-permission-answering-switch";

// The Claude Code switch and the switch for answering its permission requests in the pill. A
// flip writes nothing at first: the panel under them shows what would change in Claude Code's
// settings file, and only a confirmation writes it, and only to the file the panel showed. The
// switches always show Rust's answer, so they move only once the change is saved.
export async function fillClaudeCodeIntegrationSettingCard(integrationsCard: HTMLElement): Promise<void> {
  let shownSetting = await invoke<ClaudeCodeIntegrationSetting>(READ_CLAUDE_CODE_INTEGRATION_SETTING_COMMAND);
  const integrationSwitch = createSwitchElement(CLAUDE_CODE_INTEGRATION_SWITCH_ID);
  const permissionAnsweringSwitch = createSwitchElement(CLAUDE_CODE_PERMISSION_ANSWERING_SWITCH_ID);
  const switchChangePanel = document.createElement("div");
  switchChangePanel.className = "claude-code-switch-change-panel";
  switchChangePanel.hidden = true;
  const changeErrorElement = document.createElement("p");
  changeErrorElement.className = "setting-error";
  changeErrorElement.hidden = true;

  const showSwitches = (areSwitchesUsable: boolean): void => {
    integrationSwitch.checked = shownSetting.isIntegrationOn;
    permissionAnsweringSwitch.checked = shownSetting.isPermissionAnsweringOn;
    integrationSwitch.disabled = !areSwitchesUsable;
    // Only means something while the integration is on.
    permissionAnsweringSwitch.disabled = !areSwitchesUsable || !shownSetting.isIntegrationOn;
  };
  const closeSwitchChangePanel = (): void => {
    switchChangePanel.hidden = true;
    showSwitches(true);
  };
  const showChangeError = (changeError: unknown): void => {
    console.error("Crest could not change the Claude Code integration:", changeError);
    changeErrorElement.textContent = String(changeError);
    changeErrorElement.hidden = false;
    closeSwitchChangePanel();
  };

  const confirmSwitchChange = (switchChange: ClaudeCodeSwitchChange, confirmedPreview: ClaudeCodeSwitchChangePreview): void => {
    switchChangePanel.hidden = true;
    const expectedSettingsFileFingerprint = confirmedPreview.settingsFileFingerprint;
    const changeRequest =
      switchChange === "integrationOn" || switchChange === "integrationOff"
        ? invoke<ClaudeCodeSwitchChangeOutcome>(CHANGE_CLAUDE_CODE_INTEGRATION_SETTING_COMMAND, {
            shouldIntegrationBeOn: switchChange === "integrationOn",
            expectedSettingsFileFingerprint,
          })
        : invoke<ClaudeCodeSwitchChangeOutcome>(CHANGE_CLAUDE_CODE_PERMISSION_ANSWERING_SETTING_COMMAND, {
            shouldPermissionAnsweringBeOn: switchChange === "answeringOn",
            expectedSettingsFileFingerprint,
          });
    changeRequest
      .then((changeOutcome) => {
        if (changeOutcome.outcome === "changed") {
          shownSetting = changeOutcome.currentSetting;
          closeSwitchChangePanel();
        } else {
          offerSwitchChange(switchChange, changeOutcome.currentPreview, true);
        }
      })
      .catch(showChangeError);
  };

  const offerSwitchChange = (
    switchChange: ClaudeCodeSwitchChange,
    shownPreview: ClaudeCodeSwitchChangePreview,
    isShownAgainAfterFileChange: boolean,
  ): void => {
    const changesSomething =
      shownPreview.addedCrestHookCount + shownPreview.removedCrestHookCount + shownPreview.handedBackRequestCount > 0;
    // Nothing to read, nothing to confirm; but a rewritten file is always shown, so a file
    // that keeps changing can't loop.
    if (!changesSomething && !isShownAgainAfterFileChange) {
      confirmSwitchChange(switchChange, shownPreview);
      return;
    }
    fillClaudeCodeSwitchChangePanel(
      switchChangePanel,
      {
        switchChange,
        preview: shownPreview,
        claudeCodeSettingsFilePath: shownSetting.claudeCodeSettingsFilePath,
        isShownAgainAfterFileChange,
      },
      () => confirmSwitchChange(switchChange, shownPreview),
      closeSwitchChangePanel,
    );
    switchChangePanel.hidden = false;
  };

  const startSwitchChange = (switchChange: ClaudeCodeSwitchChange): void => {
    // The flipped switch goes back until the change is confirmed and saved.
    showSwitches(false);
    changeErrorElement.hidden = true;
    invoke<ClaudeCodeSwitchChangePreview>(PREVIEW_CLAUDE_CODE_SWITCH_CHANGE_COMMAND, { switchChange })
      .then((shownPreview) => offerSwitchChange(switchChange, shownPreview, false))
      .catch(showChangeError);
  };
  integrationSwitch.addEventListener("change", () =>
    startSwitchChange(integrationSwitch.checked ? "integrationOn" : "integrationOff"),
  );
  permissionAnsweringSwitch.addEventListener("change", () =>
    startSwitchChange(permissionAnsweringSwitch.checked ? "answeringOn" : "answeringOff"),
  );
  showSwitches(true);

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
    switchChangePanel,
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
