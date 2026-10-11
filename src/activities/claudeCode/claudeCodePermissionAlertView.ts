import "../../styles/claudeCodePermissionAlertView.css";

import { CLAUDE_CODE_PERMISSION_BUTTON_ARMING_DELAY_MILLISECONDS } from "../../frontendConstants";
import {
  CLAUDE_CODE_ACTIVITY_KIND,
  CLAUDE_CODE_ALLOW_PERMISSION_ACTION_PREFIX,
  CLAUDE_CODE_DENY_PERMISSION_ACTION_PREFIX,
} from "../../ipc/ipcChannelNames";
import { requestActivityAction } from "../../ipc/requestActivityAction";
import { createSvgIconElement } from "../createSvgIconElement";
import type { ClaudeCodePermissionAlert } from "./claudeCodeStatusTypes";
import { CLAUDE_CODE_TERMINAL_ICON_PATH } from "./claudeCodeTerminalIconPath";

const CLAUDE_CODE_PRODUCT_NAME = "Claude Code";
const UNKNOWN_PROJECT_FOLDER_TEXT = "Unknown folder";

export interface ClaudeCodePermissionAlertView {
  alertViewElement: HTMLElement;
  showPermissionAlert(permissionAlert: ClaudeCodePermissionAlert): void;
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

  const buttonRowElement = document.createElement("div");
  buttonRowElement.className = "claude-code-permission-buttons";
  let shownRequestId: number | null = null;
  let buttonArmingTimer: number | undefined;
  const setButtonsEnabled = (areButtonsEnabled: boolean): void => {
    denyButton.disabled = !areButtonsEnabled;
    allowButton.disabled = !areButtonsEnabled;
  };
  const createAnswerButton = (buttonText: string, actionPrefix: string): HTMLButtonElement => {
    const answerButton = document.createElement("button");
    answerButton.type = "button";
    answerButton.className = "claude-code-permission-button";
    answerButton.textContent = buttonText;
    answerButton.addEventListener("click", () => {
      if (shownRequestId === null) {
        return;
      }
      // One answer per request: a second click could otherwise reach the next request.
      window.clearTimeout(buttonArmingTimer);
      setButtonsEnabled(false);
      const permissionAction = `${actionPrefix}${shownRequestId}`;
      requestActivityAction(CLAUDE_CODE_ACTIVITY_KIND, permissionAction).catch((actionError: unknown) => {
        console.error(`Crest could not send "${permissionAction}" to Claude Code:`, actionError);
      });
    });
    return answerButton;
  };
  const denyButton = createAnswerButton("Deny", CLAUDE_CODE_DENY_PERMISSION_ACTION_PREFIX);
  const allowButton = createAnswerButton("Allow", CLAUDE_CODE_ALLOW_PERMISSION_ACTION_PREFIX);
  allowButton.classList.add("is-allow");
  buttonRowElement.append(denyButton, allowButton);
  alertViewElement.append(headerElement, descriptionElement, buttonRowElement);

  const armButtonsAfterDelay = (): void => {
    window.clearTimeout(buttonArmingTimer);
    setButtonsEnabled(false);
    buttonArmingTimer = window.setTimeout(
      () => setButtonsEnabled(true),
      CLAUDE_CODE_PERMISSION_BUTTON_ARMING_DELAY_MILLISECONDS,
    );
  };

  return {
    alertViewElement,
    armButtonsAfterDelay,
    showPermissionAlert(permissionAlert) {
      // textContent, never innerHTML: commands and folder names must not be able to inject markup.
      projectFolderElement.textContent = permissionAlert.projectFolderName || UNKNOWN_PROJECT_FOLDER_TEXT;
      const otherWaitingCount = permissionAlert.otherWaitingRequestCount;
      waitingCountElement.textContent = otherWaitingCount > 0 ? `+${otherWaitingCount} waiting` : "";
      const requestDescription = permissionAlert.requestDescription;
      descriptionElement.dataset.descriptionKind = requestDescription.descriptionKind;
      const descriptionLines =
        requestDescription.descriptionKind === "command" ? requestDescription.commandLines : [requestDescription.actionText];
      descriptionElement.replaceChildren(
        ...descriptionLines.map((descriptionLine) => {
          const descriptionLineElement = document.createElement("div");
          descriptionLineElement.className = "claude-code-permission-description-line";
          descriptionLineElement.textContent = descriptionLine;
          return descriptionLineElement;
        }),
      );
      // A new request (the next in the queue moved up) gets a fresh pause before clicks count.
      if (permissionAlert.requestId !== shownRequestId) {
        shownRequestId = permissionAlert.requestId;
        armButtonsAfterDelay();
      }
    },
  };
}
