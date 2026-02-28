#!/usr/bin/env python3
"""
Replay-to-Video: Convert a Megacity replay file into an MP4 video.

Launches the game in --replay --record mode to capture per-frame screenshots,
then stitches them into a video using ffmpeg.

Requirements:
    - Game binary built with rendering support
    - A display (X11/Wayland/macOS) — the game is NOT headless
    - ffmpeg installed and on PATH

Usage:
    python scripts/replay_to_video.py \
        --binary ./target/release/app \
        --replay /tmp/megacity_e2e_replay.json \
        --output /tmp/megacity_replay.mp4 \
        --framerate 30
"""

import argparse
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


def find_ffmpeg() -> str | None:
    """Return the path to ffmpeg if available, else None."""
    return shutil.which("ffmpeg")


def capture_frames(binary: str, replay: str, frame_dir: str) -> bool:
    """Run the game in replay+record mode and wait for it to exit.

    Returns True if the process exited successfully.
    """
    cmd = [binary, "--replay", replay, "--record", frame_dir]
    print(f"[replay_to_video] Running: {' '.join(cmd)}")
    print(f"[replay_to_video] Frames will be written to: {frame_dir}")

    try:
        result = subprocess.run(cmd, timeout=600)
        if result.returncode != 0:
            print(
                f"[replay_to_video] Game exited with code {result.returncode}",
                file=sys.stderr,
            )
            return False
        return True
    except subprocess.TimeoutExpired:
        print(
            "[replay_to_video] Game process timed out (600s limit)",
            file=sys.stderr,
        )
        return False
    except FileNotFoundError:
        print(
            f"[replay_to_video] Binary not found: {binary}",
            file=sys.stderr,
        )
        return False


def count_frames(frame_dir: str) -> int:
    """Count the number of frame_*.png files in the directory."""
    return len(list(Path(frame_dir).glob("frame_*.png")))


def stitch_video(
    frame_dir: str,
    output: str,
    framerate: int,
    ffmpeg: str,
) -> bool:
    """Use ffmpeg to stitch numbered PNGs into an MP4.

    Returns True on success.
    """
    input_pattern = os.path.join(frame_dir, "frame_%05d.png")
    cmd = [
        ffmpeg,
        "-y",
        "-framerate", str(framerate),
        "-i", input_pattern,
        "-c:v", "libx264",
        "-pix_fmt", "yuv420p",
        output,
    ]
    print(f"[replay_to_video] Encoding: {' '.join(cmd)}")

    try:
        result = subprocess.run(
            cmd, capture_output=True, text=True, timeout=300,
        )
        if result.returncode != 0:
            print(
                f"[replay_to_video] ffmpeg failed (exit {result.returncode}):",
                file=sys.stderr,
            )
            print(result.stderr, file=sys.stderr)
            return False
        return True
    except subprocess.TimeoutExpired:
        print("[replay_to_video] ffmpeg timed out (300s limit)", file=sys.stderr)
        return False


def main():
    parser = argparse.ArgumentParser(
        description="Convert a Megacity replay into an MP4 video",
    )
    parser.add_argument(
        "--binary", default="./target/release/app",
        help="Path to game binary (default: ./target/release/app)",
    )
    parser.add_argument(
        "--replay", required=True,
        help="Path to the replay JSON file",
    )
    parser.add_argument(
        "--output", default="/tmp/megacity_replay.mp4",
        help="Output video path (default: /tmp/megacity_replay.mp4)",
    )
    parser.add_argument(
        "--framerate", type=int, default=30,
        help="Video framerate (default: 30)",
    )
    parser.add_argument(
        "--keep-frames", action="store_true",
        help="Do not delete the temporary frame directory after encoding",
    )
    args = parser.parse_args()

    # Validate inputs
    if not Path(args.binary).exists():
        print(
            f"ERROR: Game binary not found at '{args.binary}'.\n"
            "Build it first with: cargo build --release -p app",
            file=sys.stderr,
        )
        sys.exit(1)

    if not Path(args.replay).exists():
        print(
            f"ERROR: Replay file not found at '{args.replay}'.",
            file=sys.stderr,
        )
        sys.exit(1)

    ffmpeg = find_ffmpeg()
    if ffmpeg is None:
        print(
            "ERROR: ffmpeg is not installed or not on PATH.\n"
            "Install it: brew install ffmpeg / apt install ffmpeg",
            file=sys.stderr,
        )
        sys.exit(1)

    # Create temp dir for frames
    frame_dir = tempfile.mkdtemp(prefix="megacity_frames_")
    print(f"[replay_to_video] Temp frame dir: {frame_dir}")

    try:
        # Phase 1: Capture frames
        if not capture_frames(args.binary, args.replay, frame_dir):
            print("ERROR: Frame capture failed.", file=sys.stderr)
            sys.exit(1)

        frame_count = count_frames(frame_dir)
        print(f"[replay_to_video] Captured {frame_count} frames")

        if frame_count == 0:
            print("ERROR: No frames were captured.", file=sys.stderr)
            sys.exit(1)

        # Phase 2: Stitch into video
        if not stitch_video(frame_dir, args.output, args.framerate, ffmpeg):
            print("ERROR: Video encoding failed.", file=sys.stderr)
            sys.exit(1)

        output_size = Path(args.output).stat().st_size
        print(f"[replay_to_video] Video saved: {args.output}")
        print(
            f"[replay_to_video] Size: {output_size / 1024 / 1024:.1f} MB, "
            f"{frame_count} frames at {args.framerate} fps = "
            f"{frame_count / args.framerate:.1f}s"
        )

    finally:
        if not args.keep_frames:
            print(f"[replay_to_video] Cleaning up frames: {frame_dir}")
            shutil.rmtree(frame_dir, ignore_errors=True)
        else:
            print(f"[replay_to_video] Keeping frames at: {frame_dir}")


if __name__ == "__main__":
    main()
