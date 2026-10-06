import type { PillActivity, PillArrangement } from "./pillArrangementTypes";

// What fills the pill right now: the alert while there is one, else the main activity.
// Same rule as `PillArrangement::leading_activity` in Rust, which the visibility rules use.
export function findLeadingActivity(pillArrangement: PillArrangement): PillActivity | null {
  return pillArrangement.alertActivity ?? pillArrangement.mainActivity;
}
