// The "Allowed players" card: the every-player switch, the list, and adding open players.

import { invoke } from "@tauri-apps/api/core";

import {
  CHANGE_ALLOWED_PLAYERS_COMMAND,
  CHANGE_SHOW_EVERY_PLAYER_COMMAND,
  LIST_ALLOWED_PLAYER_OPTIONS_COMMAND,
} from "../ipc/ipcChannelNames";
import { createSettingRowElement } from "./settingRowElement";

// Match `MediaPlayerDescription` / `AllowedPlayerOptions` in
// src-tauri/src/settings_window/allowed_player_options.rs.
interface MediaPlayerDescription {
  appIdentifier: string;
  label: string;
}

interface AllowedPlayerOptions {
  showEveryPlayer: boolean;
  allowedPlayers: MediaPlayerDescription[];
  playersToAdd: MediaPlayerDescription[];
}

const ADD_PLAYER_SELECT_ID = "add-player-select";
const SHOW_EVERY_PLAYER_SWITCH_ID = "show-every-player-switch";

// Fills the "Allowed players" card and redraws it from Rust's answer after every change,
// so it always shows what was really saved. Players only show up while they're open, so the
// card is also refreshed whenever the window comes back to the front (e.g. after starting
// a player).
export async function fillAllowedPlayersSettingCard(allowedPlayersCard: HTMLElement): Promise<void> {
  const showAllowedPlayerOptions = (allowedPlayerOptions: AllowedPlayerOptions): void => {
    const allowedAppIdentifiers = allowedPlayerOptions.allowedPlayers.map((allowedPlayer) => allowedPlayer.appIdentifier);
    const sendPlayerSettingChange = (commandName: string, commandArguments: Record<string, unknown>): void => {
      invoke<AllowedPlayerOptions>(commandName, commandArguments)
        .then(showAllowedPlayerOptions)
        .catch((changeError: unknown) => console.error("Crest could not change the allowed players:", changeError));
    };
    const saveAllowedPlayers = (newAllowedAppIdentifiers: string[]): void =>
      sendPlayerSettingChange(CHANGE_ALLOWED_PLAYERS_COMMAND, { allowedAppIdentifierFragments: newAllowedAppIdentifiers });
    allowedPlayersCard.replaceChildren(
      createShowEveryPlayerRow(allowedPlayerOptions.showEveryPlayer, (shouldShowEveryPlayer) =>
        sendPlayerSettingChange(CHANGE_SHOW_EVERY_PLAYER_COMMAND, { shouldShowEveryPlayer }),
      ),
      ...createAllowedPlayerRows(allowedPlayerOptions.allowedPlayers, (removedAppIdentifier) =>
        saveAllowedPlayers(allowedAppIdentifiers.filter((appIdentifier) => appIdentifier !== removedAppIdentifier)),
      ),
      createAddPlayerRow(allowedPlayerOptions.playersToAdd, (addedAppIdentifier) =>
        saveAllowedPlayers([...allowedAppIdentifiers, addedAppIdentifier]),
      ),
    );
  };
  const refreshAllowedPlayersCard = async (): Promise<void> => {
    showAllowedPlayerOptions(await invoke<AllowedPlayerOptions>(LIST_ALLOWED_PLAYER_OPTIONS_COMMAND));
  };
  await refreshAllowedPlayersCard();
  window.addEventListener("focus", () => {
    refreshAllowedPlayersCard().catch((listError: unknown) =>
      console.error("Crest could not list the players:", listError),
    );
  });
}

function createShowEveryPlayerRow(
  isShowingEveryPlayer: boolean,
  onShowEveryPlayerChange: (shouldShowEveryPlayer: boolean) => void,
): HTMLElement {
  const showEveryPlayerSwitch = document.createElement("input");
  showEveryPlayerSwitch.type = "checkbox";
  showEveryPlayerSwitch.id = SHOW_EVERY_PLAYER_SWITCH_ID;
  showEveryPlayerSwitch.className = "setting-switch";
  showEveryPlayerSwitch.setAttribute("role", "switch");
  showEveryPlayerSwitch.checked = isShowingEveryPlayer;
  showEveryPlayerSwitch.addEventListener("change", () => onShowEveryPlayerChange(showEveryPlayerSwitch.checked));
  return createSettingRowElement(
    "Show every player",
    "Ignore the list below and show whatever is playing.",
    showEveryPlayerSwitch,
  );
}

function createAllowedPlayerRows(
  allowedPlayers: MediaPlayerDescription[],
  onRemovePlayer: (appIdentifier: string) => void,
): HTMLElement[] {
  if (allowedPlayers.length === 0) {
    const noPlayerNote = document.createElement("span");
    return [
      createSettingRowElement(
        "No players chosen",
        "Unless every player is shown, the pill shows no music until you add one.",
        noPlayerNote,
      ),
    ];
  }
  return allowedPlayers.map((allowedPlayer) => {
    const removePlayerButton = document.createElement("button");
    removePlayerButton.type = "button";
    removePlayerButton.className = "setting-button";
    removePlayerButton.textContent = "Remove";
    removePlayerButton.addEventListener("click", () => onRemovePlayer(allowedPlayer.appIdentifier));
    return createSettingRowElement(allowedPlayer.label, allowedPlayer.appIdentifier, removePlayerButton);
  });
}

function createAddPlayerRow(
  playersToAdd: MediaPlayerDescription[],
  onAddPlayer: (appIdentifier: string) => void,
): HTMLElement {
  const addPlayerSelect = document.createElement("select");
  addPlayerSelect.id = ADD_PLAYER_SELECT_ID;
  addPlayerSelect.className = "setting-select";
  const placeholderOption = document.createElement("option");
  placeholderOption.value = "";
  placeholderOption.textContent = playersToAdd.length > 0 ? "Choose a player…" : "No other player is open";
  addPlayerSelect.append(placeholderOption);
  for (const playerToAdd of playersToAdd) {
    const playerOption = document.createElement("option");
    playerOption.value = playerToAdd.appIdentifier;
    playerOption.textContent = playerToAdd.label;
    addPlayerSelect.append(playerOption);
  }
  addPlayerSelect.disabled = playersToAdd.length === 0;
  addPlayerSelect.addEventListener("change", () => {
    if (addPlayerSelect.value) {
      onAddPlayer(addPlayerSelect.value);
    }
  });
  return createSettingRowElement(
    "Add a player",
    "Players appear here while they're open. Videos in normal browser tabs count as one player per browser.",
    addPlayerSelect,
  );
}
