import "../../styles/coverProgressGauge.css";

import { createAlbumArtImage } from "./albumArtImage";
import type { PlaybackPosition } from "./playbackPositionTicker";
import type { AlbumArtRingColors } from "./sampleAlbumArtRingColors";

const SVG_NAMESPACE = "http://www.w3.org/2000/svg";
const PAUSED_GAUGE_CLASS = "is-paused";
const RING_COLOR_CSS_VARIABLES = [
  "--cover-gauge-ring-start-color",
  "--cover-gauge-ring-middle-color",
  "--cover-gauge-ring-end-color",
];

export interface CoverGaugeGeometry {
  boxSize: number;
  coverSize: number;
  ringRadius: number;
  ringWidth: number;
}

export interface CoverProgressGauge {
  gaugeElement: HTMLElement;
  showAlbumArt(albumArtDataUrl: string | null, isAlbumArtLoading: boolean): void;
  showIsPlaying(isPlaying: boolean): void;
  /** The song's colours for the ring; null = the music colour (designTokens.css). */
  showRingColors(albumArtRingColors: AlbumArtRingColors | null): void;
  drawPlaybackPosition(playbackPosition: PlaybackPosition): void;
}

// Each gauge's gradient needs its own id: several gauges are in the page at once.
let createdGaugeCount = 0;

// The cover in a ring that shows how far the song is (design spec, Gauges): the ring starts at
// 12 o'clock and grows clockwise in the song's colours. It replaces the playing bars and the
// straight progress bar, and only moves when its owner redraws it (playbackPositionTicker.ts).
export function createCoverProgressGauge(coverGaugeGeometry: CoverGaugeGeometry): CoverProgressGauge {
  const { boxSize, coverSize, ringRadius, ringWidth } = coverGaugeGeometry;
  const gaugeElement = document.createElement("div");
  gaugeElement.className = "cover-progress-gauge";
  gaugeElement.style.setProperty("--cover-gauge-box-size", `${boxSize}px`);
  gaugeElement.style.setProperty("--cover-gauge-cover-size", `${coverSize}px`);

  createdGaugeCount += 1;
  const ringGradientId = `cover-gauge-ring-gradient-${createdGaugeCount}`;
  const ringCircumference = 2 * Math.PI * ringRadius;
  const boxCentre = boxSize / 2;
  const gaugeSvgElement = document.createElementNS(SVG_NAMESPACE, "svg");
  gaugeSvgElement.setAttribute("viewBox", `0 0 ${boxSize} ${boxSize}`);
  gaugeSvgElement.setAttribute("aria-hidden", "true");
  const gradientElement = document.createElementNS(SVG_NAMESPACE, "linearGradient");
  gradientElement.id = ringGradientId;
  gradientElement.setAttribute("x2", "1");
  gradientElement.setAttribute("y2", "1");
  RING_COLOR_CSS_VARIABLES.forEach((ringColorCssVariable, stopIndex) => {
    const gradientStopElement = document.createElementNS(SVG_NAMESPACE, "stop");
    gradientStopElement.setAttribute("offset", String(stopIndex / (RING_COLOR_CSS_VARIABLES.length - 1)));
    gradientStopElement.style.setProperty("stop-color", `var(${ringColorCssVariable})`);
    gradientElement.append(gradientStopElement);
  });
  const definitionsElement = document.createElementNS(SVG_NAMESPACE, "defs");
  definitionsElement.append(gradientElement);
  const createRingCircle = (className: string): SVGCircleElement => {
    const circleElement = document.createElementNS(SVG_NAMESPACE, "circle");
    circleElement.setAttribute("class", className);
    circleElement.setAttribute("cx", String(boxCentre));
    circleElement.setAttribute("cy", String(boxCentre));
    circleElement.setAttribute("r", String(ringRadius));
    circleElement.setAttribute("stroke-width", String(ringWidth));
    return circleElement;
  };
  const trackCircleElement = createRingCircle("cover-progress-track");
  const progressCircleElement = createRingCircle("cover-progress-ring");
  progressCircleElement.setAttribute("stroke", `url(#${ringGradientId})`);
  // Turned a quarter back, so the ring starts at 12 o'clock; the dash is as long as the progress.
  progressCircleElement.setAttribute("transform", `rotate(-90 ${boxCentre} ${boxCentre})`);
  progressCircleElement.setAttribute("stroke-dasharray", `${ringCircumference} ${ringCircumference}`);
  progressCircleElement.setAttribute("stroke-dashoffset", String(ringCircumference));
  gaugeSvgElement.append(definitionsElement, trackCircleElement, progressCircleElement);

  const albumArt = createAlbumArtImage();
  gaugeElement.append(gaugeSvgElement, albumArt.albumArtElement);

  return {
    gaugeElement,
    showAlbumArt(albumArtDataUrl, isAlbumArtLoading) {
      albumArt.showAlbumArt(albumArtDataUrl, isAlbumArtLoading);
    },
    showIsPlaying(isPlaying) {
      gaugeElement.classList.toggle(PAUSED_GAUGE_CLASS, !isPlaying);
    },
    showRingColors(albumArtRingColors) {
      RING_COLOR_CSS_VARIABLES.forEach((ringColorCssVariable, colorIndex) => {
        if (albumArtRingColors) {
          gaugeElement.style.setProperty(ringColorCssVariable, albumArtRingColors[colorIndex]);
        } else {
          gaugeElement.style.removeProperty(ringColorCssVariable);
        }
      });
    },
    drawPlaybackPosition(playbackPosition) {
      // A live stream has no position: the ring stays empty, only its faint track shows.
      const progressFraction = playbackPosition
        ? playbackPosition.positionMilliseconds / playbackPosition.durationMilliseconds
        : 0;
      progressCircleElement.setAttribute("stroke-dashoffset", String(ringCircumference * (1 - progressFraction)));
      progressCircleElement.style.visibility = progressFraction > 0 ? "visible" : "hidden";
    },
  };
}
