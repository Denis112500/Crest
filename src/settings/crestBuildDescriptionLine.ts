// The About card: which Crest is running (version, build, time, path).

import { invoke } from "@tauri-apps/api/core";

import { READ_CREST_BUILD_DESCRIPTION_COMMAND } from "../ipc/ipcChannelNames";

// Matches `CrestBuildDescription` in src-tauri/src/settings_window/crest_build_description.rs.
interface CrestBuildDescription {
  version: string;
  isDevelopmentBuild: boolean;
  builtAtUnixSeconds: number | null;
  executablePath: string | null;
}

const BUILD_TIME_FORMAT: Intl.DateTimeFormatOptions = { dateStyle: "medium", timeStyle: "short" };

// "Crest 0.3.0 · release build · built 5 Oct 2026, 00:29", with the running file underneath,
// so builds that share a version number can be told apart.
export async function fillCrestBuildDescriptionLine(aboutCard: HTMLElement): Promise<void> {
  const crestBuildDescription = await invoke<CrestBuildDescription>(READ_CREST_BUILD_DESCRIPTION_COMMAND);
  const buildKind = crestBuildDescription.isDevelopmentBuild ? "dev build" : "release build";
  const buildTime =
    crestBuildDescription.builtAtUnixSeconds === null
      ? "build time unknown"
      : `built ${new Date(crestBuildDescription.builtAtUnixSeconds * 1000).toLocaleString(undefined, BUILD_TIME_FORMAT)}`;
  const buildSummaryElement = document.createElement("p");
  buildSummaryElement.className = "settings-about-summary";
  buildSummaryElement.textContent = `Crest ${crestBuildDescription.version} · ${buildKind} · ${buildTime}`;
  const executablePathElement = document.createElement("p");
  executablePathElement.className = "setting-row-description";
  executablePathElement.textContent = crestBuildDescription.executablePath ?? "";
  aboutCard.replaceChildren(buildSummaryElement, executablePathElement);
}
