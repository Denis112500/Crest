import { invoke } from "@tauri-apps/api/core";

import { SHOW_CLAUDE_CODE_SETTINGS_COPIES_COMMAND } from "../ipc/ipcChannelNames";
import type { ClaudeCodeSettingsCopies } from "./claudeCodeSwitchChangeTypes";
import { createSettingRowElement } from "./settingRowElement";

interface ClaudeCodeSettingsCopiesRow {
  rowElement: HTMLElement;
  showSettingsCopies: (settingsCopies: ClaudeCodeSettingsCopies) => void;
}

// The row under the Claude Code switches about the copies Crest keeps of Claude Code's settings
// file before each change: how many, the newest one's time in the user's own time zone (the
// file names hold UTC), and a button that opens their folder.
export function createClaudeCodeSettingsCopiesRow(onShowCopiesError: (showError: unknown) => void): ClaudeCodeSettingsCopiesRow {
  const showCopiesButton = document.createElement("button");
  showCopiesButton.type = "button";
  showCopiesButton.className = "setting-button";
  showCopiesButton.textContent = "Show the copies";
  showCopiesButton.addEventListener("click", () => {
    invoke(SHOW_CLAUDE_CODE_SETTINGS_COPIES_COMMAND).catch(onShowCopiesError);
  });
  const rowElement = createSettingRowElement("Copies of Claude Code's settings", "", showCopiesButton);
  const rowDescription = rowElement.querySelector<HTMLElement>(".setting-row-description");

  const showSettingsCopies = (settingsCopies: ClaudeCodeSettingsCopies): void => {
    const keptCopiesLine =
      settingsCopies.newestCopyUnixMilliseconds === null
        ? "None yet."
        : `${settingsCopies.keptCopyCount} kept, the newest from ${new Date(settingsCopies.newestCopyUnixMilliseconds).toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" })}.`;
    if (rowDescription) {
      rowDescription.textContent = `Before each change, Crest keeps a copy of the file as it was (the newest ${settingsCopies.keptCopyLimit}). ${keptCopiesLine}`;
    }
  };
  return { rowElement, showSettingsCopies };
}
