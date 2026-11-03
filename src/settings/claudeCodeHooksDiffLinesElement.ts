import type { TextLineDiffEntry } from "./claudeCodeSwitchChangeTypes";

// A "+" or "-" in front of each changed line, as in a git diff, so the change reads without
// relying on color alone.
const DIFF_LINE_MARKS = { unchanged: "  ", added: "+ ", removed: "- " } as const;

// The exact lines of Claude Code's settings file a switch changes. Built with text nodes only:
// the lines come from a file, and are never parsed as markup.
export function createClaudeCodeHooksDiffLinesElement(hooksDiffLines: TextLineDiffEntry[]): HTMLElement {
  const diffLinesElement = document.createElement("pre");
  diffLinesElement.className = "setting-code-preview";
  for (const diffLine of hooksDiffLines) {
    const diffLineElement = document.createElement("span");
    diffLineElement.className = `setting-diff-line setting-diff-line-${diffLine.kind}`;
    diffLineElement.textContent =
      diffLine.kind === "skippedUnchanged"
        ? `  … ${diffLine.skippedLineCount} unchanged line${diffLine.skippedLineCount === 1 ? "" : "s"}`
        : `${DIFF_LINE_MARKS[diffLine.kind]}${diffLine.lineText}`;
    diffLinesElement.append(diffLineElement);
  }
  return diffLinesElement;
}
