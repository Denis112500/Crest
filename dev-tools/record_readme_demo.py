"""Records the pill for the README: an animated GIF and a still PNG of the open pill.

Captures only a small rectangle around the pill's window (top center of the main display),
so nothing else on the screen ends up in the images. The mouse pointer isn't captured.
Unchanged frames are merged, which keeps the GIF small.

    python dev-tools/record_readme_demo.py --seconds 12 --output-folder docs

Put something calm behind the pill first (e.g. an empty, maximized Notepad), because the
transparent corners around the notch show whatever is behind it.
"""

import argparse
import pathlib
import time

from PIL import Image, ImageChops, ImageGrab

# The pill window is 412 x 184 physical pixels at 100 % scaling, centered at the top of the
# main display (measured with inspect_pill_window.ps1); a small margin shows the screen edge.
PILL_WINDOW_WIDTH = 412
PILL_WINDOW_HEIGHT = 184
CAPTURE_SIDE_MARGIN = 24
CAPTURE_BOTTOM_MARGIN = 16
FRAMES_PER_SECOND = 20
GIF_FILE_NAME = "crest-demo.gif"
STILL_FILE_NAME = "crest-expanded.png"
# Pixels at least this bright (255 = white) count as the pill's text and icons when picking the
# still: the open pill shows the most of them. (Counting dark pixels failed on a dark wallpaper:
# it picked an empty capsule from the middle of the opening animation.)
PILL_CONTENT_BRIGHTNESS_THRESHOLD = 200


def calculate_capture_rectangle(main_display_width: int) -> tuple[int, int, int, int]:
    pill_left = (main_display_width - PILL_WINDOW_WIDTH) // 2
    return (
        pill_left - CAPTURE_SIDE_MARGIN,
        0,
        pill_left + PILL_WINDOW_WIDTH + CAPTURE_SIDE_MARGIN,
        PILL_WINDOW_HEIGHT + CAPTURE_BOTTOM_MARGIN,
    )


def record_frames(capture_rectangle, recording_seconds: float) -> list[tuple[Image.Image, int]]:
    """Frames with how long each one stays on screen (ms); repeated frames are merged."""
    frame_interval = 1 / FRAMES_PER_SECOND
    recorded_frames: list[tuple[Image.Image, int]] = []
    recording_end = time.perf_counter() + recording_seconds
    previous_capture_time = time.perf_counter()
    while time.perf_counter() < recording_end:
        captured_frame = ImageGrab.grab(bbox=capture_rectangle).convert("RGB")
        capture_time = time.perf_counter()
        elapsed_milliseconds = round((capture_time - previous_capture_time) * 1000)
        previous_capture_time = capture_time
        if recorded_frames and ImageChops.difference(captured_frame, recorded_frames[-1][0]).getbbox() is None:
            last_frame, last_duration = recorded_frames[-1]
            recorded_frames[-1] = (last_frame, last_duration + elapsed_milliseconds)
        else:
            recorded_frames.append((captured_frame, max(elapsed_milliseconds, round(frame_interval * 1000))))
        time.sleep(max(0.0, frame_interval - (time.perf_counter() - capture_time)))
    return recorded_frames


def count_bright_content_pixels(frame: Image.Image) -> int:
    """The open pill is the frame with the most bright pixels: title, artist, times, buttons."""
    brightness_histogram = frame.convert("L").histogram()
    return sum(brightness_histogram[PILL_CONTENT_BRIGHTNESS_THRESHOLD:])


def main() -> None:
    argument_parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    argument_parser.add_argument("--seconds", type=float, default=12)
    argument_parser.add_argument("--output-folder", default="docs")
    parsed_arguments = argument_parser.parse_args()

    main_display_width = ImageGrab.grab().size[0]
    capture_rectangle = calculate_capture_rectangle(main_display_width)
    output_folder = pathlib.Path(parsed_arguments.output_folder)
    output_folder.mkdir(parents=True, exist_ok=True)

    print(f"Recording {parsed_arguments.seconds} s of the rectangle {capture_rectangle} ...", flush=True)
    recorded_frames = record_frames(capture_rectangle, parsed_arguments.seconds)

    gif_frames = [frame for frame, _ in recorded_frames]
    frame_durations = [duration for _, duration in recorded_frames]
    gif_path = output_folder / GIF_FILE_NAME
    gif_frames[0].save(
        gif_path, save_all=True, append_images=gif_frames[1:], duration=frame_durations, loop=0, optimize=True
    )
    still_path = output_folder / STILL_FILE_NAME
    max(gif_frames, key=count_bright_content_pixels).save(still_path, optimize=True)
    print(f"{len(gif_frames)} distinct frames -> {gif_path} ({gif_path.stat().st_size // 1024} KB), still -> {still_path}")


if __name__ == "__main__":
    main()
