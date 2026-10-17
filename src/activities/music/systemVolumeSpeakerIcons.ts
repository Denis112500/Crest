import { createSvgIconElement } from "../createSvgIconElement";

export type SpeakerIconLevel = "muted" | "low" | "high";

// Crest's own speaker on the 24 × 24 grid: a box with its cone, then one or two sound waves
// (bands around the cone's mouth), or a cross for muted.
const SPEAKER_BODY_PATH = "M3 9h4l5-4v14l-5-4H3z";
const SMALL_SOUND_WAVE_PATH = "M15.5 8.5a5 5 0 0 1 0 7l-1.4-1.4a3 3 0 0 0 0-4.2z";
const LARGE_SOUND_WAVE_PATH = "M18.4 5.6a9 9 0 0 1 0 12.8l-1.4-1.4a7 7 0 0 0 0-10z";
const MUTED_CROSS_PATH = "M14.6 10l1.4-1.4 2 2 2-2 1.4 1.4-2 2 2 2-1.4 1.4-2-2-2 2-1.4-1.4 2-2z";

// All three icons go into the speaker button; CSS shows the one matching its
// `data-speaker-icon-level`, like the play / pause button does.
export function createSystemVolumeSpeakerIcons(): SVGSVGElement[] {
  const iconPathsByLevel: Record<SpeakerIconLevel, string> = {
    muted: SPEAKER_BODY_PATH + MUTED_CROSS_PATH,
    low: SPEAKER_BODY_PATH + SMALL_SOUND_WAVE_PATH,
    high: SPEAKER_BODY_PATH + SMALL_SOUND_WAVE_PATH + LARGE_SOUND_WAVE_PATH,
  };
  return Object.entries(iconPathsByLevel).map(([speakerIconLevel, iconPath]) => {
    const speakerIconElement = createSvgIconElement(iconPath);
    speakerIconElement.classList.add(`speaker-icon-${speakerIconLevel}`);
    return speakerIconElement;
  });
}
