#!/usr/bin/env python3
"""
E2E Smoke Test: LLM plays Megacity and produces a verifiable replay.

Validates the full LLM gameplay flow:
1. Launch game in --agent mode
2. LLM plays N turns via OpenRouter API
3. Save and validate the replay file
4. (Optional) Convert replay to video via ffmpeg

Usage:
    export OPENROUTER_API_KEY=sk-or-...
    python scripts/e2e_smoke_test.py --binary ./target/release/app --turns 20

Requirements:
    - Python 3.10+
    - requests (pip install requests)
    - Game binary with --agent mode support
    - (Optional) ffmpeg for video generation
"""

import argparse
import json
import logging
import os
import shutil
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass, field
from pathlib import Path

import requests

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(message)s",
    datefmt="%H:%M:%S",
)
log = logging.getLogger("e2e_smoke")

OPENROUTER_URL = "https://openrouter.ai/api/v1/chat/completions"
DEFAULT_MODEL = "anthropic/claude-sonnet-4.5"

SYSTEM_PROMPT = """\
You are playing Megacity, a 256x256 grid city builder.
Each turn you receive city stats and respond with building actions.

Available actions (respond with JSON array):
- {"PlaceRoadLine": {"start": [x,y], "end": [x,y], "road_type": "Avenue"}}
- {"ZoneRect": {"min": [x,y], "max": [x,y], "zone_type": "ResidentialLow"}}
- {"PlaceUtility": {"pos": [x,y], "utility_type": "PowerPlant"}}
- {"PlaceUtility": {"pos": [x,y], "utility_type": "WaterTower"}}
- {"PlaceService": {"pos": [x,y], "service_type": "FireStation"}}

Strategy: Build an avenue, place PowerPlant and WaterTower near it, \
zone residential and commercial along it.

Respond with ONLY a JSON array of actions. Example:
[{"PlaceRoadLine": {"start": [120,128], "end": [140,128], "road_type": "Avenue"}}, \
{"PlaceUtility": {"pos": [118,128], "utility_type": "PowerPlant"}}]
"""


# ---------------------------------------------------------------------------
# Data
# ---------------------------------------------------------------------------

@dataclass
class SmokeStats:
    """Accumulated statistics for the smoke test session."""

    turns_played: int = 0
    actions_sent: int = 0
    actions_succeeded: int = 0
    actions_failed: int = 0
    llm_errors: int = 0
    final_treasury: float = 0.0
    final_population: int = 0
    final_happiness: float = 0.0
    replay_path: str = ""
    replay_valid: bool = False
    replay_entry_count: int = 0
    start_time: float = 0.0
    elapsed: float = 0.0
    video_path: str = ""
    video_generated: bool = False


# ---------------------------------------------------------------------------
# Game process wrapper
# ---------------------------------------------------------------------------

class GameProcess:
    """Manages the game subprocess in --agent mode."""

    def __init__(self, binary: str, seed: int | None = None):
        if not Path(binary).exists():
            raise FileNotFoundError(
                f"Game binary not found at '{binary}'. "
                "Build it first with: cargo build --release -p app"
            )
        cmd = [binary, "--agent"]
        if seed is not None:
            cmd.extend(["--seed", str(seed)])
        log.info("Starting game: %s", " ".join(cmd))
        self.proc = subprocess.Popen(
            cmd,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
            bufsize=1,
        )
        # Consume the "ready" handshake message before sending commands.
        ready_line = self.proc.stdout.readline()
        if not ready_line:
            raise ConnectionError("Game process closed stdout before ready")
        ready = json.loads(ready_line)
        if ready.get("type") != "ready":
            raise RuntimeError(f"Expected 'ready' handshake, got: {ready}")

    def send(self, command: dict) -> dict:
        """Send a JSON command and read the JSON response."""
        line = json.dumps(command, separators=(",", ":"))
        log.debug(">>> %s", line)
        self.proc.stdin.write(line + "\n")
        self.proc.stdin.flush()
        response_line = self.proc.stdout.readline()
        if not response_line:
            raise ConnectionError("Game process closed stdout unexpectedly")
        log.debug("<<< %s", response_line.strip())
        return json.loads(response_line)

    def observe(self) -> dict:
        return self.send({"cmd": "observe"})

    def act(self, action: dict) -> dict:
        return self.send({"cmd": "act", "action": action})

    def step(self, ticks: int) -> dict:
        return self.send({"cmd": "step", "ticks": ticks})

    def new_game(self, seed: int) -> dict:
        return self.send({"cmd": "new_game", "seed": seed})

    def save_replay(self, path: str) -> dict:
        return self.send({"cmd": "save_replay", "path": path})

    def quit(self):
        try:
            self.send({"cmd": "quit"})
        except (ConnectionError, BrokenPipeError):
            pass
        try:
            self.proc.terminate()
            self.proc.wait(timeout=5)
        except Exception:
            self.proc.kill()

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.quit()


# ---------------------------------------------------------------------------
# LLM helpers
# ---------------------------------------------------------------------------

def call_llm(api_key: str, model: str, messages: list[dict]) -> str:
    """Call OpenRouter API and return the assistant content string."""
    headers = {
        "Authorization": f"Bearer {api_key}",
        "Content-Type": "application/json",
        "HTTP-Referer": "https://github.com/dzautner/megacity",
        "X-Title": "Megacity E2E Smoke Test",
    }
    payload = {
        "model": model,
        "messages": messages,
        "temperature": 0.7,
        "max_tokens": 2048,
    }
    for attempt in range(3):
        try:
            resp = requests.post(
                OPENROUTER_URL, headers=headers, json=payload, timeout=60,
            )
            resp.raise_for_status()
            return resp.json()["choices"][0]["message"]["content"]
        except (requests.RequestException, KeyError, json.JSONDecodeError) as exc:
            log.warning("LLM API attempt %d failed: %s", attempt + 1, exc)
            if attempt < 2:
                time.sleep(2 ** attempt)
    raise RuntimeError("OpenRouter API failed after 3 attempts")


def parse_llm_actions(content: str) -> list[dict]:
    """Parse the LLM response into a list of action dicts.

    Handles: raw JSON arrays, markdown-fenced JSON, JSON embedded in prose.
    """
    text = content.strip()

    # Strip markdown code fences
    if text.startswith("```"):
        lines = text.split("\n")
        lines = [l for l in lines if not l.strip().startswith("```")]
        text = "\n".join(lines).strip()

    # Try direct parse
    try:
        parsed = json.loads(text)
        if isinstance(parsed, list):
            return parsed
        if isinstance(parsed, dict) and "actions" in parsed:
            return parsed["actions"]
    except json.JSONDecodeError:
        pass

    # Try to find JSON array in text
    start = text.find("[")
    end = text.rfind("]")
    if start != -1 and end > start:
        try:
            parsed = json.loads(text[start : end + 1])
            if isinstance(parsed, list):
                return parsed
        except json.JSONDecodeError:
            pass

    # Try to find JSON object with "actions" key
    start = text.find("{")
    end = text.rfind("}")
    if start != -1 and end > start:
        try:
            parsed = json.loads(text[start : end + 1])
            if isinstance(parsed, dict) and "actions" in parsed:
                return parsed["actions"]
        except json.JSONDecodeError:
            pass

    log.warning("Could not parse LLM response as actions: %s", text[:200])
    return []


def format_observation(obs: dict) -> str:
    """Format a city observation into a concise summary for the LLM."""
    parts = [f"Turn observation (tick {obs.get('tick', '?')}):"]

    # Economy
    treasury = obs.get("treasury", 0)
    income = obs.get("monthly_income", 0)
    expenses = obs.get("monthly_expenses", 0)
    parts.append(
        f"Treasury: ${treasury:,.0f} | "
        f"Income: ${income:,.0f} | "
        f"Expenses: ${expenses:,.0f}"
    )

    # Population
    pop = obs.get("population", {})
    total = pop.get("total", 0)
    employed = pop.get("employed", 0)
    parts.append(f"Population: {total} (employed: {employed})")

    # Happiness
    hap = obs.get("happiness", {})
    overall = hap.get("overall", 0)
    parts.append(f"Happiness: {overall:.1f}/100")

    # Coverage
    power_cov = obs.get("power_coverage", 0)
    water_cov = obs.get("water_coverage", 0)
    parts.append(f"Power: {power_cov:.0%} | Water: {water_cov:.0%}")

    # Zone demand
    zd = obs.get("zone_demand", {})
    parts.append(
        f"Demand -> R: {zd.get('residential', 0):.0f} "
        f"C: {zd.get('commercial', 0):.0f} "
        f"I: {zd.get('industrial', 0):.0f} "
        f"O: {zd.get('office', 0):.0f}"
    )

    # Warnings
    warnings = obs.get("warnings", [])
    if warnings:
        parts.append("Warnings: " + "; ".join(warnings[:5]))

    parts.append("Respond with a JSON array of actions.")
    return "\n".join(parts)


# ---------------------------------------------------------------------------
# Phase 1: Play the game
# ---------------------------------------------------------------------------

def play_game(args: argparse.Namespace, api_key: str) -> SmokeStats:
    """Launch the game, play N turns with LLM, save replay, return stats."""
    stats = SmokeStats(start_time=time.time(), replay_path=args.replay_path)

    with GameProcess(args.binary, seed=args.seed) as game:
        # Initialize new game
        log.info("Sending new_game with seed %d", args.seed)
        game.new_game(args.seed)

        conversation: list[dict] = [
            {"role": "system", "content": SYSTEM_PROMPT},
        ]

        for turn in range(1, args.turns + 1):
            log.info("=== Turn %d/%d ===", turn, args.turns)

            # Step simulation
            try:
                game.step(args.ticks_per_turn)
            except Exception as exc:
                log.error("step failed on turn %d: %s", turn, exc)
                break

            # Observe
            try:
                obs_response = game.observe()
                observation = obs_response.get("observation", obs_response)
            except Exception as exc:
                log.error("observe failed on turn %d: %s", turn, exc)
                break

            # Track city state
            stats.final_treasury = observation.get("treasury", 0)
            pop = observation.get("population", {})
            stats.final_population = pop.get("total", 0)
            hap = observation.get("happiness", {})
            stats.final_happiness = hap.get("overall", 0)

            # Format for LLM
            user_msg = format_observation(observation)

            # Keep conversation manageable: system + last 4 exchanges + new
            if len(conversation) > 9:
                conversation = [conversation[0]] + conversation[-8:]
            conversation.append({"role": "user", "content": user_msg})

            # Call LLM
            try:
                response_text = call_llm(api_key, args.model, conversation)
                conversation.append(
                    {"role": "assistant", "content": response_text}
                )
            except Exception as exc:
                log.error("LLM call failed on turn %d: %s", turn, exc)
                stats.llm_errors += 1
                stats.turns_played = turn
                continue

            # Parse actions
            actions = parse_llm_actions(response_text)
            log.info("LLM returned %d action(s)", len(actions))

            # Execute actions
            for action in actions:
                stats.actions_sent += 1
                try:
                    result = game.act(action)
                    if result.get("result") == "Success":
                        stats.actions_succeeded += 1
                        log.info("  OK: %s", _summarize(action))
                    else:
                        stats.actions_failed += 1
                        log.warning(
                            "  FAIL: %s -> %s",
                            _summarize(action),
                            result.get("error", result.get("result", "?")),
                        )
                except Exception as exc:
                    stats.actions_failed += 1
                    log.error("  ERROR sending action: %s", exc)

            stats.turns_played = turn

            # Early exit if bankrupt
            if stats.final_treasury < -100_000:
                log.warning("City is deeply bankrupt, stopping early")
                break

        # Save replay
        log.info("Saving replay to %s", args.replay_path)
        try:
            game.save_replay(args.replay_path)
            log.info("Replay saved successfully")
        except Exception as exc:
            log.error("Failed to save replay: %s", exc)

    stats.elapsed = time.time() - stats.start_time
    return stats


def _summarize(action: dict) -> str:
    """Return a short summary string for an action dict."""
    if isinstance(action, dict):
        for key, val in action.items():
            return f"{key}({json.dumps(val, separators=(',', ':'))})"
    return str(action)[:80]


# ---------------------------------------------------------------------------
# Phase 2: Validate the replay
# ---------------------------------------------------------------------------

def validate_replay(path: str) -> tuple[bool, int, list[str]]:
    """Validate the replay file at *path*.

    Returns (is_valid, entry_count, issues).
    """
    issues: list[str] = []

    if not Path(path).exists():
        return False, 0, ["Replay file does not exist"]

    try:
        raw = Path(path).read_text()
    except Exception as exc:
        return False, 0, [f"Could not read replay file: {exc}"]

    try:
        data = json.loads(raw)
    except json.JSONDecodeError as exc:
        return False, 0, [f"Replay file is not valid JSON: {exc}"]

    if not isinstance(data, dict):
        issues.append(f"Expected top-level dict, got {type(data).__name__}")
        return False, 0, issues

    # Check for expected top-level keys
    for key in ("header", "entries"):
        if key not in data:
            issues.append(f"Missing top-level key: '{key}'")

    entries = data.get("entries", [])
    if not isinstance(entries, list):
        issues.append("'entries' is not a list")
        entries = []

    entry_count = len(entries)

    # If there is an entry_count field (in header or footer), verify match
    header = data.get("header", {})
    footer = data.get("footer", {})
    declared_count = (
        footer.get("entry_count")
        or header.get("entry_count")
        or data.get("entry_count")
    )
    if declared_count is not None and declared_count != entry_count:
        issues.append(
            f"Declared entry_count ({declared_count}) != "
            f"actual entries length ({entry_count})"
        )

    if entry_count == 0:
        issues.append("Replay has zero entries")

    is_valid = len(issues) == 0
    return is_valid, entry_count, issues


# ---------------------------------------------------------------------------
# Phase 3: Replay to video (optional)
# ---------------------------------------------------------------------------

def generate_video(
    binary: str,
    replay_path: str,
    video_path: str,
    framerate: int = 30,
) -> tuple[bool, str]:
    """Convert a replay to video using the game's --record mode + ffmpeg.

    Returns (success, video_path_or_error_message).
    """
    ffmpeg = shutil.which("ffmpeg")
    if ffmpeg is None:
        return False, "ffmpeg not found on PATH"

    frame_dir = tempfile.mkdtemp(prefix="megacity_frames_")
    log.info("Capturing frames to: %s", frame_dir)

    try:
        # Run game in replay+record mode
        cmd = [binary, "--replay", replay_path, "--record", frame_dir]
        log.info("Running: %s", " ".join(cmd))
        result = subprocess.run(cmd, timeout=600)
        if result.returncode != 0:
            return False, f"Game exited with code {result.returncode}"

        # Count captured frames
        frames = sorted(Path(frame_dir).glob("frame_*.png"))
        if len(frames) == 0:
            return False, "No frames were captured"

        log.info("Captured %d frames", len(frames))

        # Stitch with ffmpeg
        input_pattern = os.path.join(frame_dir, "frame_%05d.png")
        ffmpeg_cmd = [
            ffmpeg,
            "-y",
            "-framerate", str(framerate),
            "-i", input_pattern,
            "-c:v", "libx264",
            "-pix_fmt", "yuv420p",
            video_path,
        ]
        log.info("Encoding: %s", " ".join(ffmpeg_cmd))
        result = subprocess.run(
            ffmpeg_cmd, capture_output=True, text=True, timeout=300,
        )
        if result.returncode != 0:
            return False, f"ffmpeg failed: {result.stderr[:500]}"

        size_mb = Path(video_path).stat().st_size / 1024 / 1024
        duration = len(frames) / framerate
        log.info(
            "Video saved: %s (%.1f MB, %.1fs at %d fps)",
            video_path, size_mb, duration, framerate,
        )
        return True, video_path

    except subprocess.TimeoutExpired:
        return False, "Process timed out"
    except Exception as exc:
        return False, str(exc)
    finally:
        shutil.rmtree(frame_dir, ignore_errors=True)


# ---------------------------------------------------------------------------
# Report
# ---------------------------------------------------------------------------

def print_report(
    stats: SmokeStats,
    replay_valid: bool,
    replay_entries: int,
    replay_issues: list[str],
):
    """Print a human-readable summary of the smoke test."""
    ok = (
        stats.turns_played > 0
        and stats.actions_succeeded > 0
        and replay_valid
    )
    status = "PASS" if ok else "FAIL"

    print()
    print("=" * 60)
    print(f"  E2E SMOKE TEST: {status}")
    print("=" * 60)
    print(f"  Turns played:       {stats.turns_played}")
    print(f"  Actions sent:       {stats.actions_sent}")
    print(f"    Succeeded:        {stats.actions_succeeded}")
    print(f"    Failed:           {stats.actions_failed}")
    print(f"  LLM errors:         {stats.llm_errors}")
    print(f"  Duration:           {stats.elapsed:.1f}s")
    print()
    print(f"  Final treasury:     ${stats.final_treasury:,.0f}")
    print(f"  Final population:   {stats.final_population}")
    print(f"  Final happiness:    {stats.final_happiness:.1f}/100")
    print()
    print(f"  Replay file:        {stats.replay_path}")
    print(f"  Replay valid:       {replay_valid}")
    print(f"  Replay entries:     {replay_entries}")
    if replay_issues:
        print("  Replay issues:")
        for issue in replay_issues:
            print(f"    - {issue}")
    if stats.video_path:
        print()
        print(f"  Video file:         {stats.video_path}")
        print(f"  Video generated:    {stats.video_generated}")
    print("=" * 60)

    return ok


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(
        description="E2E smoke test: LLM plays Megacity and produces a replay",
    )
    parser.add_argument(
        "--binary", default="./target/release/app",
        help="Path to game binary (default: ./target/release/app)",
    )
    parser.add_argument(
        "--model", default=DEFAULT_MODEL,
        help=f"OpenRouter model (default: {DEFAULT_MODEL})",
    )
    parser.add_argument(
        "--seed", type=int, default=42,
        help="Game seed (default: 42)",
    )
    parser.add_argument(
        "--turns", type=int, default=20,
        help="Number of turns to play (default: 20)",
    )
    parser.add_argument(
        "--ticks-per-turn", type=int, default=100,
        help="Simulation ticks per turn (default: 100)",
    )
    parser.add_argument(
        "--replay-path", default="/tmp/megacity_e2e_replay.json",
        help="Path for the replay file (default: /tmp/megacity_e2e_replay.json)",
    )
    parser.add_argument(
        "--video-path", default="/tmp/megacity_e2e_replay.mp4",
        help="Path for the video file (default: /tmp/megacity_e2e_replay.mp4)",
    )
    parser.add_argument(
        "--no-video", action="store_true",
        help="Skip Phase 3 (replay-to-video conversion)",
    )
    parser.add_argument(
        "-v", "--verbose", action="store_true",
        help="Enable debug logging",
    )
    args = parser.parse_args()

    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)

    # Pre-flight checks
    api_key = os.environ.get("OPENROUTER_API_KEY")
    if not api_key:
        print(
            "ERROR: OPENROUTER_API_KEY environment variable is not set.\n"
            "Get a key at https://openrouter.ai/ and export it:\n"
            "  export OPENROUTER_API_KEY=sk-or-...",
            file=sys.stderr,
        )
        sys.exit(1)

    if not Path(args.binary).exists():
        print(
            f"ERROR: Game binary not found at '{args.binary}'.\n"
            "Build it first with:\n"
            "  cargo build --release -p app",
            file=sys.stderr,
        )
        sys.exit(1)

    # Phase 1: LLM plays the game
    log.info("Phase 1: LLM plays the game (%d turns)", args.turns)
    stats = play_game(args, api_key)

    # Phase 2: Validate replay
    log.info("Phase 2: Validating replay at %s", args.replay_path)
    replay_valid, replay_entries, replay_issues = validate_replay(
        args.replay_path,
    )

    # Phase 3: Replay to video (optional)
    if not args.no_video:
        ffmpeg_available = shutil.which("ffmpeg") is not None
        if ffmpeg_available and replay_valid:
            log.info(
                "Phase 3: Converting replay to video -> %s", args.video_path,
            )
            success, result = generate_video(
                args.binary, args.replay_path, args.video_path,
            )
            stats.video_generated = success
            stats.video_path = args.video_path if success else result
        elif not ffmpeg_available:
            log.info("Phase 3: Skipped (ffmpeg not available)")
            stats.video_path = "(skipped: ffmpeg not found)"
        else:
            log.info("Phase 3: Skipped (replay not valid)")
            stats.video_path = "(skipped: invalid replay)"
    else:
        log.info("Phase 3: Skipped (--no-video)")

    # Report
    ok = print_report(stats, replay_valid, replay_entries, replay_issues)

    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
