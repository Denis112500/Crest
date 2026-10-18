import "../../styles/claudeCodePermissionAlertView.css";

import { createSvgIconElement } from "../createSvgIconElement";
import { createClaudeCodePermissionAnswerButtons } from "./claudeCodePermissionAnswerButtons";
import { createClaudeCodePermissionCommandBox } from "./claudeCodePermissionCommandBox";
import { CLAUDE_CODE_PRODUCT_NAME } from "./claudeCodeProductName";
import type { ClaudeCodePermissionAlert, ClaudeCodePermissionDescription } from "./claudeCodeStatusTypes";
import { CLAUDE_CODE_TERMINAL_ICON_PATH } from "./claudeCodeTerminalIconPath";

const UNKNOWN_PROJECT_FOLDER_TEXT = "Unknown folder";
const NEW_REQUEST_HIGHLIGHT_CLASS = "is-new-request";

export interface ClaudeCodePermissionAlertView {
  alertViewElement: HTMLElement;
  /** Returns how many rows the request takes, for the open pill's height. */
  showPermissionAlert(permissionAlert: ClaudeCodePermissionAlert): number;
  /** The alert ended: its next request is a fresh start, not one that moved up. */
  forgetShownRequest(): void;
  /** Disables Allow / Deny for a moment; called whenever the alert becomes visible. */
  armButtonsAfterDelay(): void;
}

// The open pill while Claude Code asks for permission: which session, what it wants to run or
// change, and Allow / Deny. A click is an action for the Rust source with the request's id; the
// alert goes away through the next arrangement update, whichever side answered first.
export function createClaudeCodePermissionAlertView(): ClaudeCodePermissionAlertView {
  const alertViewElement = document.createElement("div");
  alertViewElement.className = "claude-code-permission-alert-view";

  const headerElement = document.createElement("div");
  headerElement.className = "claude-code-permission-header";
  const terminalIconElement = createSvgIconElement(CLAUDE_CODE_TERMINAL_ICON_PATH);
  terminalIconElement.classList.add("claude-code-terminal-icon", "claude-code-permission-icon");
  const productNameElement = document.createElement("span");
  productNameElement.className = "claude-code-permission-product";
  productNameElement.textContent = CLAUDE_CODE_PRODUCT_NAME;
  const projectFolderElement = document.createElement("span");
  projectFolderElement.className = "claude-code-permission-project";
  const waitingCountElement = document.createElement("span");
  waitingCountElement.className = "claude-code-permission-waiting-count";
  headerElement.append(terminalIconElement, productNameElement, projectFolderElement, waitingCountElement);

  const descriptionElement = document.createElement("div");
  descriptionElement.className = "claude-code-permission-description";
  descriptionElement.addEventListener("animationend", () => descriptionElement.classList.remove(NEW_REQUEST_HIGHLIGHT_CLASS));
  const actionLineElement = document.createElement("div");
  actionLineElement.className = "claude-code-permission-action-line";
  let isCommandBoxShown = false;
  const commandBox = createClaudeCodePermissionCommandBox(() => answerButtons.refreshButtons());
  const answerButtons = createClaudeCodePermissionAnswerButtons(
    () => !isCommandBoxShown || commandBox.hasWholeCommandBeenSeen(),
  );
  alertViewElement.append(headerElement, descriptionElement, answerButtons.buttonRowElement);

  // Returns the request's row count. A connected tool's call goes in the command box too: it must
  // be read to the end before Allow, like a command.
  const showRequestDescription = (requestDescription: ClaudeCodePermissionDescription): number => {
    descriptionElement.dataset.descriptionKind = requestDescription.descriptionKind;
    isCommandBoxShown = requestDescription.descriptionKind !== "action";
    if (requestDescription.descriptionKind === "action") {
      // textContent, never innerHTML: names from Claude Code must not be able to inject markup.
      actionLineElement.textContent = requestDescription.actionText;
      descriptionElement.replaceChildren(actionLineElement);
      return 1;
    }
    descriptionElement.replaceChildren(commandBox.commandBoxElement);
    return commandBox.showCommand(
      requestDescription.descriptionKind === "command" ? requestDescription.commandText : requestDescription.callText,
    );
  };

  let shownRequestId: number | null = null;
  let shownRowCount = 1;
  return {
    alertViewElement,
    armButtonsAfterDelay: answerButtons.armAfterDelay,
    forgetShownRequest() {
      shownRequestId = null;
    },
    showPermissionAlert(permissionAlert) {
      projectFolderElement.textContent = permissionAlert.projectFolderName || UNKNOWN_PROJECT_FOLDER_TEXT;
      const otherWaitingCount = permissionAlert.otherWaitingRequestCount;
      waitingCountElement.textContent = otherWaitingCount > 0 ? `+${otherWaitingCount} waiting` : "";
      // An update for the same request (another one joined the queue) leaves the box and its
      // scroll position alone. A new request gets a fresh pause before clicks count, and a flash
      // when it moved up behind one just answered.
      if (permissionAlert.requestId === shownRequestId) {
        return shownRowCount;
      }
      const didRequestMoveUp = shownRequestId !== null;
      shownRequestId = permissionAlert.requestId;
      const requestDescription = permissionAlert.requestDescription;
      shownRowCount = showRequestDescription(requestDescription);
      answerButtons.prepareForRequest(permissionAlert.requestId, isRequestCut(requestDescription));
      if (didRequestMoveUp) {
        descriptionElement.classList.remove(NEW_REQUEST_HIGHLIGHT_CLASS);
        // Next frame, so the class change restarts the animation if one is still running.
        window.requestAnimationFrame(() => descriptionElement.classList.add(NEW_REQUEST_HIGHLIGHT_CLASS));
      }
      answerButtons.armAfterDelay();
      return shownRowCount;
    },
  };
}

function isRequestCut(requestDescription: ClaudeCodePermissionDescription): boolean {
  switch (requestDescription.descriptionKind) {
    case "command":
      return requestDescription.isCommandCut;
    case "toolCall":
      return requestDescription.isCallCut;
    case "action":
      return false;
  }
}
