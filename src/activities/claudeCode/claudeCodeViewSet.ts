import type { ActivityViewSet } from "../activityViewSet";
import { calculateClaudeCodeExpandedHeight } from "./calculateClaudeCodeExpandedHeight";
import { calculateClaudeCodePermissionAlertHeight } from "./calculateClaudeCodePermissionAlertHeight";
import { createClaudeCodePermissionAlertView } from "./claudeCodePermissionAlertView";
import type { ClaudeCodeStatusPayload } from "./claudeCodeStatusTypes";
import { createCompactClaudeCodeView } from "./compactClaudeCodeView";
import { createCompanionClaudeCodeView } from "./companionClaudeCodeView";
import { createExpandedClaudeCodeView } from "./expandedClaudeCodeView";

// The "claude-code" activity's views, as registered in activityViewRegistry.ts. The open pill
// shows the session list, or the permission alert while a request waits. No animations, so
// nothing to pause while hidden; only the alert's buttons care about being seen.
export function createClaudeCodeViewSet(): ActivityViewSet {
  const compactClaudeCodeView = createCompactClaudeCodeView();
  const companionClaudeCodeView = createCompanionClaudeCodeView();
  const expandedClaudeCodeView = createExpandedClaudeCodeView();
  const permissionAlertView = createClaudeCodePermissionAlertView();
  // Holds whichever of the two open views is current; the pill's expanded layer keeps this one
  // element, so switching never needs the presenter.
  const expandedSlotElement = document.createElement("div");
  expandedSlotElement.className = "claude-code-expanded-slot";
  // Replaced by the first payload, which always comes right after the views are mounted.
  let expandedLogicalHeight = calculateClaudeCodeExpandedHeight(1, false);
  let isExpandedViewVisible = false;
  return {
    compactViewElement: compactClaudeCodeView.compactViewElement,
    companionViewElement: companionClaudeCodeView.companionViewElement,
    expandedViewElement: expandedSlotElement,
    showActivityPayload(activityPayload) {
      // Rust's Claude Code source always sends this shape (claude_code_session_list.rs).
      const claudeCodeStatus = activityPayload as ClaudeCodeStatusPayload;
      compactClaudeCodeView.showClaudeCodeStatus(claudeCodeStatus);
      companionClaudeCodeView.showClaudeCodeStatus(claudeCodeStatus);
      const permissionAlert = claudeCodeStatus.permissionAlert;
      if (permissionAlert) {
        permissionAlertView.showPermissionAlert(permissionAlert);
        showInExpandedSlot(expandedSlotElement, permissionAlertView.alertViewElement);
        const requestDescription = permissionAlert.requestDescription;
        expandedLogicalHeight = calculateClaudeCodePermissionAlertHeight(
          requestDescription.descriptionKind === "command" ? requestDescription.commandLines.length : 1,
        );
        return;
      }
      expandedClaudeCodeView.showClaudeCodeStatus(claudeCodeStatus);
      showInExpandedSlot(expandedSlotElement, expandedClaudeCodeView.expandedViewElement);
      expandedLogicalHeight = calculateClaudeCodeExpandedHeight(
        claudeCodeStatus.listedSessions.length,
        claudeCodeStatus.unlistedSessionCount > 0,
      );
    },
    expandedViewLogicalHeight: () => expandedLogicalHeight,
    // Called again after every change, so only the moment it becomes visible counts: an alert
    // that was hidden (during a game) gets its pause before clicks count when it shows again.
    setExpandedViewVisible(isVisibleNow) {
      if (isVisibleNow && !isExpandedViewVisible) {
        permissionAlertView.armButtonsAfterDelay();
      }
      isExpandedViewVisible = isVisibleNow;
    },
    setCompactPlaceOnScreen: () => {},
  };
}

function showInExpandedSlot(expandedSlotElement: HTMLElement, openViewElement: HTMLElement): void {
  if (expandedSlotElement.firstElementChild !== openViewElement) {
    expandedSlotElement.replaceChildren(openViewElement);
  }
}
