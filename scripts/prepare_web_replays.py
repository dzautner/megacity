#!/usr/bin/env python3
"""
Prepare replay JSON files for browser playback.

This tool optionally time-compresses tick spacing so long-idle replays remain
watchable in web viewer mode.
"""

from __future__ import annotations

import argparse
import json
import math
from pathlib import Path


def sanitize_filename(name: str) -> str:
    return (
        name.replace(":", "_")
        .replace("/", "_")
        .replace("\\", "_")
        .replace(" ", "_")
    )


def compress_ticks(replay: dict, target_end_tick: int) -> tuple[dict, int]:
    footer = replay.get("footer", {})
    end_tick = int(footer.get("end_tick", 0) or 0)
    if target_end_tick <= 0:
        scale = 1
    else:
        scale = max(1, math.ceil(end_tick / target_end_tick))

    entries = replay.get("entries", [])
    prev_tick = 0
    for entry in entries:
        old_tick = int(entry.get("tick", 0))
        new_tick = old_tick // scale
        if new_tick < prev_tick:
            new_tick = prev_tick
        entry["tick"] = new_tick
        prev_tick = new_tick

    replay.setdefault("header", {})["start_tick"] = 0
    replay.setdefault("footer", {})["end_tick"] = prev_tick + 1
    replay["footer"]["entry_count"] = len(entries)
    return replay, scale


def main() -> int:
    parser = argparse.ArgumentParser(description="Prepare browser replay assets")
    parser.add_argument(
        "--inputs",
        nargs="+",
        required=True,
        help="Replay JSON paths",
    )
    parser.add_argument(
        "--output-dir",
        default="crates/app/assets/replays",
        help="Output directory for prepared replay files",
    )
    parser.add_argument(
        "--target-end-tick",
        type=int,
        default=1800,
        help="Compress so resulting footer.end_tick is ~this value",
    )
    args = parser.parse_args()

    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    manifest = []
    for input_path_str in args.inputs:
        input_path = Path(input_path_str)
        with input_path.open("r", encoding="utf-8") as f:
            replay = json.load(f)

        original_end = int(replay.get("footer", {}).get("end_tick", 0) or 0)
        replay, scale = compress_ticks(replay, args.target_end_tick)
        new_end = int(replay.get("footer", {}).get("end_tick", 0) or 0)
        stem = input_path.name.replace(".replay", "")
        safe_stem = sanitize_filename(stem)
        out_name = f"{safe_stem}.web_x{scale}.replay"
        out_path = output_dir / out_name

        with out_path.open("w", encoding="utf-8") as f:
            json.dump(replay, f, separators=(",", ":"))

        manifest.append(
            {
                "name": stem,
                "safe_name": safe_stem,
                "file": out_name,
                "source": input_path.name,
                "scale": scale,
                "original_end_tick": original_end,
                "web_end_tick": new_end,
                "entry_count": int(replay.get("footer", {}).get("entry_count", 0) or 0),
            }
        )
        print(f"[ok] {input_path.name} -> {out_name} (x{scale}, end {original_end} -> {new_end})")

    manifest_path = output_dir / "index.json"
    with manifest_path.open("w", encoding="utf-8") as f:
        json.dump({"replays": manifest}, f, indent=2)
    print(f"[ok] wrote manifest: {manifest_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
