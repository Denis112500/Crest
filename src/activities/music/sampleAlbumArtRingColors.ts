import {
  ALBUM_ART_RING_MAX_LIGHTNESS,
  ALBUM_ART_RING_MIN_LIGHTNESS,
  ALBUM_ART_RING_SAMPLE_LOGICAL_HEIGHT,
  ALBUM_ART_RING_SAMPLE_LOGICAL_WIDTH,
  ALBUM_ART_RING_SATURATION_BOOST,
} from "../../frontendConstants";

/** Three CSS colours for the ring around the cover, from its left, middle and right thirds. */
export type AlbumArtRingColors = [string, string, string];

// One small canvas for every track: the cover is drawn into it once per track (never per frame).
const samplingCanvas = document.createElement("canvas");
samplingCanvas.width = ALBUM_ART_RING_SAMPLE_LOGICAL_WIDTH;
samplingCanvas.height = ALBUM_ART_RING_SAMPLE_LOGICAL_HEIGHT;

// The song's colours for the ring around its cover (the pill's edge has a fixed music colour, so
// a cover's colours can't pass for a Claude Code status): the average of the cover's left,
// middle and right thirds, made a little more vivid and kept between a minimum and maximum
// lightness, so a dark or grey cover still gives a visible ring and a white one doesn't glare.
// The cover is a data: URL from Rust, which doesn't taint the canvas, so its pixels may be read.
export async function sampleAlbumArtRingColors(albumArtDataUrl: string): Promise<AlbumArtRingColors | null> {
  const coverImage = new Image();
  coverImage.src = albumArtDataUrl;
  try {
    await coverImage.decode();
  } catch {
    return null;
  }
  const drawingContext = samplingCanvas.getContext("2d", { willReadFrequently: true });
  if (!drawingContext) {
    return null;
  }
  drawingContext.drawImage(coverImage, 0, 0, samplingCanvas.width, samplingCanvas.height);
  const pixelBytes = drawingContext.getImageData(0, 0, samplingCanvas.width, samplingCanvas.height).data;
  const thirdWidth = samplingCanvas.width / 3;
  const thirdColors = [0, 1, 2].map((thirdIndex) => {
    let redSum = 0;
    let greenSum = 0;
    let blueSum = 0;
    let pixelCount = 0;
    for (let rowIndex = 0; rowIndex < samplingCanvas.height; rowIndex += 1) {
      for (let columnIndex = thirdIndex * thirdWidth; columnIndex < (thirdIndex + 1) * thirdWidth; columnIndex += 1) {
        const byteOffset = (rowIndex * samplingCanvas.width + columnIndex) * 4;
        redSum += pixelBytes[byteOffset];
        greenSum += pixelBytes[byteOffset + 1];
        blueSum += pixelBytes[byteOffset + 2];
        pixelCount += 1;
      }
    }
    return describeAsRingColor(redSum / pixelCount, greenSum / pixelCount, blueSum / pixelCount);
  });
  return thirdColors as AlbumArtRingColors;
}

// RGB (0–255) to an HSL colour string, more saturated and with its lightness kept in range.
function describeAsRingColor(red: number, green: number, blue: number): string {
  const redShare = red / 255;
  const greenShare = green / 255;
  const blueShare = blue / 255;
  const largestShare = Math.max(redShare, greenShare, blueShare);
  const smallestShare = Math.min(redShare, greenShare, blueShare);
  const lightness = (largestShare + smallestShare) / 2;
  const chroma = largestShare - smallestShare;
  let hueDegrees = 0;
  let saturation = 0;
  if (chroma > 0) {
    saturation = chroma / (1 - Math.abs(2 * lightness - 1));
    if (largestShare === redShare) {
      hueDegrees = 60 * (((greenShare - blueShare) / chroma) % 6);
    } else if (largestShare === greenShare) {
      hueDegrees = 60 * ((blueShare - redShare) / chroma + 2);
    } else {
      hueDegrees = 60 * ((redShare - greenShare) / chroma + 4);
    }
  }
  const vividSaturation = Math.min(1, saturation * ALBUM_ART_RING_SATURATION_BOOST);
  const visibleLightness = Math.min(ALBUM_ART_RING_MAX_LIGHTNESS, Math.max(ALBUM_ART_RING_MIN_LIGHTNESS, lightness));
  const roundedHue = Math.round((hueDegrees + 360) % 360);
  return `hsl(${roundedHue} ${Math.round(vividSaturation * 100)}% ${Math.round(visibleLightness * 100)}%)`;
}
