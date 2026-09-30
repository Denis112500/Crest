import "../../styles/albumArtImage.css";

import { createSvgIconElement } from "../createSvgIconElement";

// A single eighth note: stem, flag and round head.
const MUSIC_NOTE_ICON_PATH = "M10 4h8v4h-5v9a3.5 3.5 0 1 1-3-3.46z";
const MISSING_ART_CLASS = "is-missing-art";

export interface AlbumArtImage {
  albumArtElement: HTMLElement;
  showAlbumArt(albumArtDataUrl: string | null): void;
}

// Album art with a music-note placeholder when the player sends none, or sends
// something the browser can't decode.
export function createAlbumArtImage(sizeClassName: string): AlbumArtImage {
  const albumArtElement = document.createElement("div");
  albumArtElement.classList.add("album-art", sizeClassName, MISSING_ART_CLASS);
  const albumArtPictureElement = document.createElement("img");
  albumArtPictureElement.className = "album-art-picture";
  albumArtPictureElement.alt = "";
  albumArtPictureElement.addEventListener("error", () => albumArtElement.classList.add(MISSING_ART_CLASS));
  const placeholderIconElement = createSvgIconElement(MUSIC_NOTE_ICON_PATH);
  placeholderIconElement.classList.add("album-art-placeholder");
  albumArtElement.append(placeholderIconElement, albumArtPictureElement);

  let displayedAlbumArtDataUrl: string | null = null;
  return {
    albumArtElement,
    showAlbumArt(albumArtDataUrl) {
      // Re-assigning the same data URL would make the browser decode the image again.
      if (albumArtDataUrl === displayedAlbumArtDataUrl) {
        return;
      }
      displayedAlbumArtDataUrl = albumArtDataUrl;
      if (albumArtDataUrl) {
        albumArtPictureElement.src = albumArtDataUrl;
        albumArtElement.classList.remove(MISSING_ART_CLASS);
      } else {
        albumArtPictureElement.removeAttribute("src");
        albumArtElement.classList.add(MISSING_ART_CLASS);
      }
    },
  };
}
