// What the Claude Code card and Rust exchange. Each type mirrors the Rust type named above it.

// `ClaudeCodeSwitchChange` in src-tauri/src/settings_window/claude_code_switch_change_preview.rs.
export type ClaudeCodeSwitchChange = "integrationOn" | "integrationOff" | "answeringOn" | "answeringOff";

// `ClaudeCodeSettingsCopies` in src-tauri/src/activity_sources/claude_code/claude_code_settings_backups.rs.
export interface ClaudeCodeSettingsCopies {
  keptCopyCount: number;
  keptCopyLimit: number;
  newestCopyUnixMilliseconds: number | null;
}

// `ClaudeCodeIntegrationSetting` in src-tauri/src/settings_window/claude_code_integration_commands.rs.
export interface ClaudeCodeIntegrationSetting {
  isIntegrationOn: boolean;
  isPermissionAnsweringOn: boolean;
  claudeCodeSettingsFilePath: string;
  settingsCopies: ClaudeCodeSettingsCopies;
}

// `TextLineDiffEntry` in src-tauri/src/text_line_diff.rs.
export type TextLineDiffEntry =
  | { kind: "unchanged" | "added" | "removed"; lineText: string }
  | { kind: "skippedUnchanged"; skippedLineCount: number };

// `ClaudeCodeSwitchChangePreview` in claude_code_switch_change_preview.rs, with the fields of
// `ClaudeCodeSettingsChangePreview` and `ClaudeCodeHooksChangePreview` flattened into it.
export interface ClaudeCodeSwitchChangePreview {
  addedCrestHookCount: number;
  removedCrestHookCount: number;
  keptOtherSettingCount: number;
  keptOwnHookCount: number;
  isEveryHookDisabled: boolean;
  hooksDiffLines: TextLineDiffEntry[];
  settingsFileFingerprint: string;
  handedBackRequestCount: number;
}

// `ClaudeCodeSwitchChangeOutcome` in claude_code_integration_commands.rs.
export type ClaudeCodeSwitchChangeOutcome =
  | { outcome: "changed"; currentSetting: ClaudeCodeIntegrationSetting }
  | { outcome: "settingsFileChangedSincePreview"; currentPreview: ClaudeCodeSwitchChangePreview };
