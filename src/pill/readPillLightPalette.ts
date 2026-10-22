/** Three CSS colours along the pill's edge, left to right. */
export type PillLightColorSet = [string, string, string];

export interface PillLightPalette {
  starlightColors: PillLightColorSet;
  goldColors: PillLightColorSet;
  usageLimitColors: PillLightColorSet;
  doneColors: PillLightColorSet;
  musicColors: PillLightColorSet;
  fullStrength: number;
  pausedStrength: number;
  waitingStrength: number;
}

// The light's colour sets and strengths live in designTokens.css with every other colour; the
// page reads them once at startup, after the stylesheets have loaded.
export function readPillLightPalette(): PillLightPalette {
  const rootStyle = getComputedStyle(document.documentElement);
  const readToken = (tokenName: string): string => rootStyle.getPropertyValue(tokenName).trim();
  const readColorSet = (setName: string): PillLightColorSet => [
    readToken(`--pill-light-${setName}-start-color`),
    readToken(`--pill-light-${setName}-middle-color`),
    readToken(`--pill-light-${setName}-end-color`),
  ];
  return {
    starlightColors: readColorSet("starlight"),
    goldColors: readColorSet("gold"),
    usageLimitColors: readColorSet("usage-limit"),
    doneColors: readColorSet("done"),
    musicColors: readColorSet("music"),
    fullStrength: Number.parseFloat(readToken("--pill-light-full-strength")),
    pausedStrength: Number.parseFloat(readToken("--pill-light-paused-strength")),
    waitingStrength: Number.parseFloat(readToken("--pill-light-waiting-strength")),
  };
}
