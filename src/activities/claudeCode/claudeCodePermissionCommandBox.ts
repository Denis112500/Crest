import {
  CLAUDE_CODE_COMMAND_SCROLL_END_LOGICAL_TOLERANCE,
  CLAUDE_CODE_PERMISSION_COMMAND_MAX_VISIBLE_ROWS,
} from "../../frontendConstants";
import { estimateClaudeCodeCommandRowCount } from "./estimateClaudeCodeCommandRowCount";

export interface ClaudeCodePermissionCommandBox {
  commandBoxElement: HTMLElement;
  /** Shows a new command from its top; returns how many rows the box is tall. */
  showCommand(commandText: string): number;
  /** True once every row has been on screen: it fit, or it was scrolled to the end. */
  hasWholeCommandBeenSeen(): boolean;
}

// A command waiting for permission, wrapped at the box's edge, at most 3 rows tall and scrolling
// past that. Allow depends on having seen all of it, so "seen" comes from the box's real scroll
// position, never from the row estimate. Only the user's scrolling wakes it: nothing runs idle.
export function createClaudeCodePermissionCommandBox(handleWholeCommandSeen: () => void): ClaudeCodePermissionCommandBox {
  const commandBoxElement = document.createElement("div");
  commandBoxElement.className = "claude-code-permission-command-box";
  let wasWholeCommandSeen = false;

  const isScrolledToTheEnd = (): boolean =>
    // 0 while the box isn't laid out (the pill hidden): that never counts as seen.
    commandBoxElement.clientHeight > 0 &&
    commandBoxElement.scrollTop + commandBoxElement.clientHeight >=
      commandBoxElement.scrollHeight - CLAUDE_CODE_COMMAND_SCROLL_END_LOGICAL_TOLERANCE;

  commandBoxElement.addEventListener("scroll", () => {
    if (!wasWholeCommandSeen && isScrolledToTheEnd()) {
      wasWholeCommandSeen = true;
      handleWholeCommandSeen();
    }
  });

  return {
    commandBoxElement,
    showCommand(commandText) {
      // textContent, never innerHTML: a command must not be able to inject markup.
      commandBoxElement.textContent = commandText;
      commandBoxElement.scrollTop = 0;
      wasWholeCommandSeen = false;
      const visibleRowCount = Math.min(
        estimateClaudeCodeCommandRowCount(commandText),
        CLAUDE_CODE_PERMISSION_COMMAND_MAX_VISIBLE_ROWS,
      );
      commandBoxElement.style.setProperty("--claude-code-command-visible-rows", String(visibleRowCount));
      return visibleRowCount;
    },
    hasWholeCommandBeenSeen() {
      wasWholeCommandSeen ||= isScrolledToTheEnd();
      return wasWholeCommandSeen;
    },
  };
}
