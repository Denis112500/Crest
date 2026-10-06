import { fillAllowedPlayersSettingCard } from "./allowedPlayersSettingCard";
import { fillClaudeCodeIntegrationSettingCard } from "./claudeCodeIntegrationSettingCard";
import { fillCrestBuildDescriptionLine } from "./crestBuildDescriptionLine";
import { createLaunchAtLoginSettingRow } from "./launchAtLoginSettingRow";
import { createPillDisplaySettingRow } from "./pillDisplaySettingRow";

// Fills the settings page's cards. Each row reads its current value from Rust when the
// window opens; the window is created fresh every time, so there's nothing to keep in sync.
// Every card is filled on its own: if one can't be read, the others still appear.
function startSettingsWindow(): void {
  const generalSettingsCard = document.querySelector<HTMLElement>("#general-settings-card");
  const displaySettingsCard = document.querySelector<HTMLElement>("#display-settings-card");
  const allowedPlayersCard = document.querySelector<HTMLElement>("#allowed-players-card");
  const integrationsCard = document.querySelector<HTMLElement>("#integrations-card");
  const aboutCard = document.querySelector<HTMLElement>("#about-card");
  if (!generalSettingsCard || !displaySettingsCard || !allowedPlayersCard || !integrationsCard || !aboutCard) {
    throw new Error("settings.html is missing a settings card");
  }
  const reportCardFailure = (cardName: string) => (cardError: unknown) =>
    console.error(`Crest could not fill the ${cardName} settings:`, cardError);
  createLaunchAtLoginSettingRow()
    .then((launchAtLoginSettingRow) => generalSettingsCard.append(launchAtLoginSettingRow))
    .catch(reportCardFailure("general"));
  createPillDisplaySettingRow()
    .then((pillDisplaySettingRow) => displaySettingsCard.append(pillDisplaySettingRow))
    .catch(reportCardFailure("display"));
  fillAllowedPlayersSettingCard(allowedPlayersCard).catch(reportCardFailure("player"));
  fillClaudeCodeIntegrationSettingCard(integrationsCard).catch(reportCardFailure("integrations"));
  fillCrestBuildDescriptionLine(aboutCard).catch(reportCardFailure("about"));
}

startSettingsWindow();
