import { createClaudeCodeHooksDiffLinesElement } from "./claudeCodeHooksDiffLinesElement";
import type { ClaudeCodeSwitchChange, ClaudeCodeSwitchChangePreview } from "./claudeCodeSwitchChangeTypes";

interface ShownClaudeCodeSwitchChange {
  switchChange: ClaudeCodeSwitchChange;
  preview: ClaudeCodeSwitchChangePreview;
  claudeCodeSettingsFilePath: string;
  // The user confirmed once, but Claude Code had rewritten its file since the preview.
  isShownAgainAfterFileChange: boolean;
}

const SWITCH_CHANGE_WORDING: Record<ClaudeCodeSwitchChange, { question: string; confirmLabel: string }> = {
  integrationOn: { question: "Switch on Claude Code?", confirmLabel: "Switch on" },
  integrationOff: { question: "Switch off Claude Code?", confirmLabel: "Switch off" },
  answeringOn: { question: "Answer permission requests in the pill?", confirmLabel: "Switch on" },
  answeringOff: { question: "Stop answering permission requests in the pill?", confirmLabel: "Switch off" },
};

// Shown under the Claude Code card's switches before a flip writes anything: what changes in
// Claude Code's settings file, what stays, which waiting requests go back to Claude Code, and
// the exact lines, folded. Drawn again from scratch each time, so it never shows an old preview.
export function fillClaudeCodeSwitchChangePanel(
  panelElement: HTMLElement,
  shownChange: ShownClaudeCodeSwitchChange,
  onConfirm: () => void,
  onCancel: () => void,
): void {
  const { preview } = shownChange;
  const wording = SWITCH_CHANGE_WORDING[shownChange.switchChange];
  const panelLines: HTMLElement[] = [createPanelLine("claude-code-switch-change-question", wording.question)];
  if (shownChange.isShownAgainAfterFileChange) {
    panelLines.push(
      createPanelLine(
        "claude-code-switch-change-warning",
        "Claude Code's settings file changed after this was shown, so here is the change as it is now.",
      ),
    );
  }
  panelLines.push(createPanelLine("setting-row-description", describeFileChange(preview, shownChange.claudeCodeSettingsFilePath)));
  const keptSettingsLine = describeKeptSettings(preview);
  if (keptSettingsLine) {
    panelLines.push(createPanelLine("setting-row-description", keptSettingsLine));
  }
  if (preview.handedBackRequestCount > 0) {
    const handedBackRequestCount = preview.handedBackRequestCount;
    panelLines.push(
      createPanelLine(
        "claude-code-switch-change-warning",
        handedBackRequestCount === 1
          ? "1 waiting request goes back to Claude Code's own question."
          : `${handedBackRequestCount} waiting requests go back to Claude Code's own question.`,
      ),
    );
  }
  if (preview.isEveryHookDisabled) {
    panelLines.push(
      createPanelLine(
        "claude-code-switch-change-warning",
        "disableAllHooks is on in this file, so Claude Code runs no hooks at all, Crest's included, until it's switched off there.",
      ),
    );
  }
  if (preview.hooksDiffLines.length > 0) {
    panelLines.push(
      createExactLinesDetails(preview),
      createPanelLine(
        "setting-row-description",
        "A copy of the file is kept first. Running Claude Code sessions pick the change up too. Nothing leaves this PC.",
      ),
    );
  }

  const cancelButton = createPanelButton("setting-button", "Cancel", onCancel);
  const confirmButton = createPanelButton("setting-button setting-button-confirm", wording.confirmLabel, onConfirm);
  const panelButtons = document.createElement("div");
  panelButtons.className = "claude-code-switch-change-buttons";
  panelButtons.append(cancelButton, confirmButton);
  panelElement.replaceChildren(...panelLines, panelButtons);
  panelElement.onkeydown = (keyEvent) => {
    if (keyEvent.key === "Escape") {
      onCancel();
    }
  };
  confirmButton.focus();
}

function describeFileChange(preview: ClaudeCodeSwitchChangePreview, claudeCodeSettingsFilePath: string): string {
  const { addedCrestHookCount, removedCrestHookCount } = preview;
  if (addedCrestHookCount > 0 && removedCrestHookCount > 0) {
    return `In ${claudeCodeSettingsFilePath}: removes ${countCrestHooks(removedCrestHookCount)} and adds ${countCrestHooks(addedCrestHookCount)}.`;
  }
  if (addedCrestHookCount > 0) {
    return `Adds ${countCrestHooks(addedCrestHookCount)} to ${claudeCodeSettingsFilePath}.`;
  }
  if (removedCrestHookCount > 0) {
    return `Removes ${countCrestHooks(removedCrestHookCount)} from ${claudeCodeSettingsFilePath}.`;
  }
  return `Nothing changes in ${claudeCodeSettingsFilePath}.`;
}

function describeKeptSettings(preview: ClaudeCodeSwitchChangePreview): string | undefined {
  const { keptOtherSettingCount, keptOwnHookCount } = preview;
  const keptParts = [
    keptOtherSettingCount > 0 ? `your ${keptOtherSettingCount} other setting${keptOtherSettingCount === 1 ? "" : "s"}` : "",
    keptOwnHookCount > 0 ? `your ${keptOwnHookCount} own hook${keptOwnHookCount === 1 ? "" : "s"}` : "",
  ].filter((keptPart) => keptPart !== "");
  return keptParts.length > 0 ? `It keeps ${keptParts.join(" and ")} as they are.` : undefined;
}

function countCrestHooks(crestHookCount: number): string {
  return `${crestHookCount} Crest hook${crestHookCount === 1 ? "" : "s"}`;
}

function createExactLinesDetails(preview: ClaudeCodeSwitchChangePreview): HTMLElement {
  const addedLineCount = preview.hooksDiffLines.filter((diffLine) => diffLine.kind === "added").length;
  const removedLineCount = preview.hooksDiffLines.filter((diffLine) => diffLine.kind === "removed").length;
  const exactLinesDetails = document.createElement("details");
  exactLinesDetails.className = "setting-details";
  const exactLinesSummary = document.createElement("summary");
  exactLinesSummary.textContent = `Exact lines (+${addedLineCount} −${removedLineCount})`;
  exactLinesDetails.append(exactLinesSummary, createClaudeCodeHooksDiffLinesElement(preview.hooksDiffLines));
  return exactLinesDetails;
}

function createPanelLine(lineClassName: string, lineText: string): HTMLElement {
  const panelLine = document.createElement("p");
  panelLine.className = lineClassName;
  panelLine.textContent = lineText;
  return panelLine;
}

function createPanelButton(buttonClassName: string, buttonLabel: string, onClick: () => void): HTMLButtonElement {
  const panelButton = document.createElement("button");
  panelButton.type = "button";
  panelButton.className = buttonClassName;
  panelButton.textContent = buttonLabel;
  panelButton.addEventListener("click", onClick);
  return panelButton;
}
