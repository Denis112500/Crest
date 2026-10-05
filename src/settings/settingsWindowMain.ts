import { fillAllowedPlayersSettingCard } from "./allowedPlayersSettingCard";
import { fillCrestBuildDescriptionLine } from "./crestBuildDescriptionLine";
import { createLaunchAtLoginSettingRow } from "./launchAtLoginSettingRow";
import { createPillDisplaySettingRow } from "./pillDisplaySettingRow";

// Fills the settings page's cards. Each row reads its current value from Rust when the
// window opens; the window is created fresh every time, so there's nothing to keep in sync.
async function startSettingsWindow(): Promise<void> {
  const generalSettingsCard = document.querySelector<HTMLElement>("#general-settings-card");
  const displaySettingsCard = document.querySelector<HTMLElement>("#display-settings-card");
  const allowedPlayersCard = document.querySelector<HTMLElement>("#allowed-players-card");
  const aboutCard = document.querySelector<HTMLElement>("#about-card");
  if (!generalSettingsCard || !displaySettingsCard || !allowedPlayersCard || !aboutCard) {
    throw new Error("settings.html is missing a settings card");
  }
  const [launchAtLoginSettingRow, pillDisplaySettingRow] = await Promise.all([
    createLaunchAtLoginSettingRow(),
    createPillDisplaySettingRow(),
    fillAllowedPlayersSettingCard(allowedPlayersCard),
    fillCrestBuildDescriptionLine(aboutCard),
  ]);
  generalSettingsCard.append(launchAtLoginSettingRow);
  displaySettingsCard.append(pillDisplaySettingRow);
}

startSettingsWindow().catch((startupError: unknown) => {
  console.error("Crest could not fill the settings window:", startupError);
});
