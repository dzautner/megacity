#!/usr/bin/env python3
"""
LLM Gameplay Harness for Megacity.

Connects the game's --agent mode to an LLM via OpenRouter,
enabling fully autonomous city-building gameplay sessions with
spatial awareness, conversation management, and query support.

Usage:
    export OPENROUTER_API_KEY=sk-or-...
    python scripts/llm_player.py --model anthropic/claude-sonnet-4-5-20250929 --seed 42 --turns 100

Requirements:
    - Python 3.10+
    - requests (pip install requests)
    - Game binary with --agent mode support
"""

import argparse
import json
import logging
import os
import subprocess
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path

import requests

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(message)s",
    datefmt="%H:%M:%S",
)
log = logging.getLogger("llm_player")

OPENROUTER_URL = "https://openrouter.ai/api/v1/chat/completions"
DEFAULT_MODEL = "anthropic/claude-sonnet-4.5"
DEFAULT_TICKS_PER_TURN = 1500
DEFAULT_MAX_TURNS = 200
PROTOCOL_VERSION = 1

SYSTEM_PROMPT = """\
You are playing Megacity, a 256x256 grid city builder. Each turn you receive stats and respond with JSON actions.

## CRITICAL RULES
1. People need BOTH housing AND jobs. Zone Residential AND Commercial/Industrial every turn.
2. Utilities MUST be ON a road cell. Utilities on grass = zero coverage.
3. If you get BlockedByWater errors, STOP going that direction. Try north, south, or west instead.
4. Use LARGE zone rects (e.g. 5x10) not tiny 1x1 zones. More zones = more buildings = more people.

## Building Pattern
For each new block: Road → Utility ON road → Zone rect on BOTH sides of road.

Example: Build a road from (100,80) to (120,80), place PowerPlant at (105,80) and WaterTower at (110,80),
then zone residential (100,81) to (120,85) on one side and commercial (100,75) to (120,79) on the other side.

## Actions (respond with ONLY a JSON object, no other text)
{"actions": [
  {"PlaceRoadLine": {"start": [x,y], "end": [x,y], "road_type": "Local"}},
  {"ZoneRect": {"min": [x,y], "max": [x,y], "zone_type": "ResidentialLow"}},
  {"PlaceUtility": {"pos": [x,y], "utility_type": "PowerPlant"}},
  {"PlaceService": {"pos": [x,y], "service_type": "FireStation"}},
  {"SetTaxRates": {"residential": 0.09, "commercial": 0.09, "industrial": 0.09, "office": 0.09}}
]}

## Zone Types
ResidentialLow, ResidentialMedium, ResidentialHigh, CommercialLow, CommercialHigh, Industrial, Office, MixedUse

## Utilities (MUST be ON road cells!)
PowerPlant ($1000, range 30), WaterTower ($200, range 25)

## Services
FireStation ($800), PoliceStation ($600), Hospital ($2000), ElementarySchool ($2000), HighSchool ($1500)

## Key Facts
- Zone rects must be adjacent to a road (within 2 cells)
- Buildings need power AND water to grow (from utilities on the road network)
- Water (~) is unbuildable. Rivers and lakes exist inside the map.
- Starting treasury: $50,000
"""


@dataclass
class SessionStats:
    turns_played: int = 0
    actions_sent: int = 0
    actions_succeeded: int = 0
    actions_failed: int = 0
    llm_errors: int = 0
    treasury_history: list = field(default_factory=list)
    population_history: list = field(default_factory=list)
    happiness_history: list = field(default_factory=list)
    start_time: float = 0.0


class ConversationManager:
    """Manages LLM conversation history with summarization."""

    def __init__(self, system_prompt: str):
        self.system_prompt = system_prompt
        self.history_summary = ""
        self.recent_turns: list[tuple[str, str]] = []  # (observation, response)
        self.turn_log: list[dict] = []

    def build_messages(self, current_observation: str) -> list[dict]:
        """Build the message list for the LLM API call."""
        messages = [{"role": "system", "content": self.system_prompt}]
        if self.history_summary:
            messages.append({
                "role": "system",
                "content": f"## Your History\n{self.history_summary}",
            })
        for obs, response in self.recent_turns[-5:]:
            messages.append({"role": "user", "content": obs})
            messages.append({"role": "assistant", "content": response})
        messages.append({"role": "user", "content": current_observation})
        return messages

    def record_turn(
        self, turn: int, observation: str, response: str, results: list,
        treasury: float = 0, population: int = 0,
    ):
        """Record a completed turn and compress history periodically."""
        self.recent_turns.append((observation, response))
        if len(self.recent_turns) > 6:
            self.recent_turns.pop(0)
        self.turn_log.append({
            "turn": turn,
            "obs_snippet": observation[:200],
            "results": results,
            "treasury": treasury,
            "population": population,
        })
        if turn > 0 and turn % 10 == 0:
            self._compress_history(turn)

    def _compress_history(self, current_turn: int):
        """Summarize the last 10 turns into a compact history block."""
        block_start = max(1, current_turn - 9)
        block = self.turn_log[block_start - 1 : current_turn]
        actions_taken = sum(len(t.get("results", [])) for t in block)
        successes = sum(
            sum(1 for r in t.get("results", []) if r.get("success", False))
            for t in block
        )
        failures = actions_taken - successes
        # Include metrics from last turn in block
        last = block[-1] if block else {}
        treasury = last.get("treasury", "?")
        pop = last.get("population", "?")
        summary = (
            f"Turns {block_start}-{current_turn}: "
            f"{actions_taken} actions ({successes} ok, {failures} failed). "
            f"Treasury=${treasury}, Pop={pop}."
        )
        self.history_summary += summary + "\n"


class GameProcess:
    """Manages the game subprocess in --agent mode."""

    def __init__(self, binary: str, seed: int | None = None):
        cmd = [binary, "--agent"]
        if seed is not None:
            cmd.extend(["--seed", str(seed)])
        log.info("Starting game: %s", " ".join(cmd))
        self.proc = subprocess.Popen(
            cmd,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
        )
        # The game sends a "ready" message on startup — consume it so
        # subsequent send/recv pairs stay aligned.
        ready_line = self.proc.stdout.readline()
        if ready_line:
            ready = json.loads(ready_line)
            log.info("Game ready (protocol v%s)", ready.get("protocol_version", "?"))

    def send(self, command: dict) -> dict:
        """Send a JSON command and read the JSON response."""
        line = json.dumps(command, separators=(",", ":"))
        log.debug(">>> %s", line)
        self.proc.stdin.write(line + "\n")
        self.proc.stdin.flush()

        response_line = self.proc.stdout.readline()
        if not response_line:
            raise ConnectionError("Game process closed stdout")
        log.debug("<<< %s", response_line.strip())
        return json.loads(response_line)

    def observe(self) -> dict:
        return self.send({"cmd": "observe"})

    def act(self, action: dict) -> dict:
        return self.send({"cmd": "act", "action": action})

    def batch_act(self, actions: list[dict]) -> dict:
        return self.send({"cmd": "batch_act", "actions": actions})

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
        self.proc.terminate()
        self.proc.wait(timeout=5)

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.quit()


class LLMClient:
    """OpenRouter API client."""

    def __init__(self, model: str, api_key: str, temperature: float = 0.7):
        self.model = model
        self.api_key = api_key
        self.temperature = temperature

    def call(self, messages: list[dict]) -> str:
        """Send messages to LLM and return the response content string."""
        response = self._call_api(messages)
        return response["choices"][0]["message"]["content"]

    def _call_api(self, messages: list[dict]) -> dict:
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
            "HTTP-Referer": "https://github.com/dzautner/megacity",
            "X-Title": "Megacity LLM Player",
        }
        payload = {
            "model": self.model,
            "messages": messages,
            "temperature": self.temperature,
            "max_tokens": 4096,
        }

        for attempt in range(8):
            try:
                resp = requests.post(
                    OPENROUTER_URL, headers=headers, json=payload, timeout=120,
                )
                resp.raise_for_status()
                return resp.json()
            except (requests.RequestException, json.JSONDecodeError) as e:
                log.warning("API attempt %d failed: %s", attempt + 1, e)
                if attempt < 7:
                    # Exponential backoff: 2, 4, 8, 16, 32, 60, 60 seconds
                    delay = min(2 ** (attempt + 1), 60)
                    log.info("  retrying in %ds...", delay)
                    time.sleep(delay)
        raise RuntimeError("OpenRouter API failed after 8 attempts")


def parse_overview_map(overview_map: str) -> list:
    """Parse overview map string into grid lines (64 strings, each char = 4x4 block)."""
    if not overview_map:
        return []
    lines = overview_map.strip().split("\n")
    grid_lines = []
    for line in lines[1:]:  # skip column header
        if not line.strip():
            break  # stop at legend separator
        if "|" in line:
            _, _, content = line.partition("|")
            grid_lines.append(content.rstrip())
    return grid_lines


def build_water_set(grid_lines: list) -> set:
    """Build a set of grid coordinates known to be water from the overview map."""
    water = set()
    scale = 4  # overview is 64x64 for 256x256 grid
    for oy, row in enumerate(grid_lines):
        for ox, ch in enumerate(row):
            if ch == '~':
                for dy in range(scale):
                    for dx in range(scale):
                        water.add((ox * scale + dx, oy * scale + dy))
    return water


def compute_buildable_area(grid_lines: list) -> str:
    """Find safe buildable area from parsed overview grid lines."""
    if not grid_lines:
        return ""

    # Find bounds of buildable (non-water) cells
    min_x, min_y = 999, 999
    max_x, max_y = 0, 0
    for y, row in enumerate(grid_lines):
        for x, ch in enumerate(row):
            if ch != '~' and ch != ' ':  # Not water or padding
                min_x = min(min_x, x)
                max_x = max(max_x, x)
                min_y = min(min_y, y)
                max_y = max(max_y, y)
    if min_x > max_x:
        return ""
    # Scale from overview coords (64x64) to grid coords (256x256)
    scale = 4
    gx0, gy0 = min_x * scale, min_y * scale
    gx1, gy1 = (max_x + 1) * scale - 1, (max_y + 1) * scale - 1
    # Clamp to grid bounds
    gx0 = max(0, min(gx0, 255))
    gy0 = max(0, min(gy0, 255))
    gx1 = max(0, min(gx1, 255))
    gy1 = max(0, min(gy1, 255))
    # Find a safe center point (well away from water edges)
    margin = 5 * scale  # 5 overview cells = 20 grid cells margin
    sx0, sy0 = gx0 + margin, gy0 + margin
    sx1, sy1 = gx1 - margin, gy1 - margin
    cx, cy = (sx0 + sx1) // 2, (sy0 + sy1) // 2
    # Cap center to safe range
    cx = max(sx0, min(cx, sx1))
    cy = max(sy0, min(cy, sy1))
    return (f"Buildable land: ({sx0},{sy0}) to ({sx1},{sy1}). Center: ({cx},{cy}). "
            f"Build roads starting near the center. "
            f"NOTE: Rivers/lakes exist INSIDE this area — if you hit water, try a different direction.")


# Cache for buildable area (set on turn 1)
_buildable_area_info = ""

# Set of known water grid coordinates — built from overview map + runtime failures
_water_cells: set = set()

# Track consecutive water failures to help LLM change direction
_water_fail_positions: list = []
_consecutive_water_turns = 0


def _action_coords(action: dict) -> list:
    """Extract all grid coordinate pairs from an action."""
    coords = []
    for _key, params in action.items():
        if isinstance(params, dict):
            for ck in ["start", "end", "pos", "min", "max"]:
                v = params.get(ck)
                if isinstance(v, list) and len(v) >= 2:
                    coords.append((int(v[0]), int(v[1])))
            # For road lines, check intermediate points
            start = params.get("start")
            end = params.get("end")
            if isinstance(start, list) and isinstance(end, list):
                x0, y0 = int(start[0]), int(start[1])
                x1, y1 = int(end[0]), int(end[1])
                if x0 == x1:  # vertical
                    for y in range(min(y0, y1), max(y0, y1) + 1):
                        coords.append((x0, y))
                elif y0 == y1:  # horizontal
                    for x in range(min(x0, x1), max(x0, x1) + 1):
                        coords.append((x, y0))
    return coords


def _action_hits_water(action: dict) -> bool:
    """Check if any coordinate of this action is in known water."""
    if not _water_cells:
        return False
    return any(c in _water_cells for c in _action_coords(action))


def _action_out_of_bounds(action: dict) -> bool:
    """Check if any coordinate of this action is outside the 256x256 grid."""
    for x, y in _action_coords(action):
        if x < 0 or x >= 256 or y < 0 or y >= 256:
            return True
    return False


def _record_water_failure(action: dict):
    """Add coordinates from a water-blocked action to the water set."""
    for coord in _action_coords(action):
        _water_cells.add(coord)
        # Also mark surrounding cells as likely water (water comes in patches)
        x, y = coord
        for dx in range(-2, 3):
            for dy in range(-2, 3):
                _water_cells.add((x + dx, y + dy))


def format_observation(obs: dict, turn: int = 0) -> str:
    """Format a city observation into a compact summary for the LLM."""
    parts = [f"## Turn {turn}"]

    # Economy
    treasury = obs.get("treasury", 0)
    income = obs.get("monthly_income", 0)
    expenses = obs.get("monthly_expenses", 0)
    parts.append(f"Treasury: ${treasury:,.0f} | Income: ${income:,.0f}/mo | Expenses: ${expenses:,.0f}/mo")

    # Population
    pop = obs.get("population", {})
    total = pop.get("total", 0)
    employed = pop.get("employed", 0)
    unemployed = pop.get("unemployed", 0)
    homeless = pop.get("homeless", 0)
    parts.append(f"Population: {total} | Employed: {employed} | Unemployed: {unemployed} | Homeless: {homeless}")

    # Happiness
    hap = obs.get("happiness", {})
    happiness = hap.get("overall", 0)
    parts.append(f"Happiness: {happiness:.1f}/100")

    # Coverage (infrastructure + services)
    power_cov = obs.get("power_coverage", 0)
    water_cov = obs.get("water_coverage", 0)
    svcs = obs.get("services", {})
    parts.append(
        f"Coverage -- Power: {power_cov:.0%} | Water: {water_cov:.0%} | "
        f"Fire: {svcs.get('fire', 0):.0%} | Police: {svcs.get('police', 0):.0%} | "
        f"Health: {svcs.get('health', 0):.0%} | Education: {svcs.get('education', 0):.0%}"
    )

    # Buildings and attractiveness
    bldgs = obs.get("building_count", 0)
    attract = obs.get("attractiveness_score", 0)
    parts.append(f"Buildings: {bldgs} | Attractiveness: {attract:.1f}/100 (need >60 for immigration)")

    # Attractiveness breakdown (if available)
    attr_bd = obs.get("attractiveness", {})
    if attr_bd:
        parts.append(
            f"  Attract breakdown -- Employment: {attr_bd.get('employment', 0):.0%} | "
            f"Happiness: {attr_bd.get('happiness', 0):.0%} | "
            f"Services: {attr_bd.get('services', 0):.0%} | "
            f"Housing: {attr_bd.get('housing', 0):.0%} | "
            f"Tax: {attr_bd.get('tax', 0):.0%}"
        )

    # Zone demand with explicit guidance
    zd = obs.get("zone_demand", {})
    r_dem = zd.get("residential", 0)
    c_dem = zd.get("commercial", 0)
    i_dem = zd.get("industrial", 0)
    o_dem = zd.get("office", 0)
    parts.append(f"Demand -- R:{r_dem:.0f} C:{c_dem:.0f} I:{i_dem:.0f} O:{o_dem:.0f}")
    # Highlight high-demand zones the LLM should prioritize
    high_demand = []
    if c_dem > 50:
        high_demand.append(f"Commercial ({c_dem:.0f}%)")
    if i_dem > 50:
        high_demand.append(f"Industrial ({i_dem:.0f}%)")
    if o_dem > 50:
        high_demand.append(f"Office ({o_dem:.0f}%)")
    if r_dem > 50:
        high_demand.append(f"Residential ({r_dem:.0f}%)")
    if high_demand:
        parts.append(f"ACTION NEEDED: High demand for {', '.join(high_demand)} — zone these to grow!")

    # Recent action results (from game engine)
    recent_results = obs.get("recent_action_results", [])
    failed_results = [r for r in recent_results if not r.get("success", True)]
    if failed_results:
        parts.append("FAILED ACTIONS (last turn):")
        for r in failed_results[-5:]:
            parts.append(f"  - {r.get('action_summary', '?')}")

    # Warnings
    warnings = obs.get("warnings", [])
    if warnings:
        parts.append(f"WARNINGS: {', '.join(warnings)}")

    # Proactive hints based on city state
    if total > 100 and happiness < 40:
        missing = []
        if svcs.get('fire', 0) < 0.5:
            missing.append("FireStation")
        if svcs.get('police', 0) < 0.5:
            missing.append("PoliceStation")
        if svcs.get('health', 0) < 0.5:
            missing.append("Hospital")
        if svcs.get('education', 0) < 0.5:
            missing.append("ElementarySchool")
        if missing:
            parts.append(f"TIP: Happiness is low! Place services near your buildings: {', '.join(missing)}")

    # Buildable area (cached from terrain query)
    global _buildable_area_info
    if _buildable_area_info:
        parts.append(_buildable_area_info)

    # Water failure tracking
    global _consecutive_water_turns, _water_fail_positions
    if _water_fail_positions:
        total_water_fails = len(_water_fail_positions)
        if total_water_fails > 3:
            # Show water-blocked regions so LLM avoids them
            recent = _water_fail_positions[-10:]
            coords_str = ", ".join(f"({x},{y})" for x, y in recent)
            parts.append(
                f"WATER BLOCKED: {total_water_fails} total failures. Recent water hits: {coords_str}. "
                f"AVOID these areas — they are rivers/lakes."
            )
    if _consecutive_water_turns >= 2:
        if _water_fail_positions:
            avg_x = sum(p[0] for p in _water_fail_positions[-6:]) // min(6, len(_water_fail_positions))
            avg_y = sum(p[1] for p in _water_fail_positions[-6:]) // min(6, len(_water_fail_positions))
            parts.append(
                f"WARNING: You hit water {_consecutive_water_turns} turns in a row near ({avg_x},{avg_y})! "
                f"STOP building in this direction. Expand in a DIFFERENT direction."
            )

    return "\n".join(parts)


def format_layers(layer_response: dict) -> str:
    """Format query layer responses into text for the LLM."""
    parts = ["## Layer Query Results"]
    layers = layer_response.get("layers", layer_response)
    if isinstance(layers, dict):
        for name, data in layers.items():
            parts.append(f"\n### {name}")
            if isinstance(data, str):
                parts.append(data)
            else:
                parts.append(json.dumps(data, indent=2)[:3000])
    elif isinstance(layers, str):
        parts.append(layers)
    else:
        parts.append(str(layers)[:3000])
    return "\n".join(parts)


def parse_response(content: str) -> dict:
    """Parse LLM response as either {"actions": [...]} or {"query": [...]}."""
    import re
    text = content.strip()

    # Strip markdown fences (```json ... ```)
    text = re.sub(r'```\w*\s*', '', text).strip()
    text = text.rstrip('`').strip()

    # Try direct parse
    try:
        parsed = json.loads(text)
        if isinstance(parsed, dict):
            return parsed
        if isinstance(parsed, list):
            return {"actions": parsed}
    except json.JSONDecodeError:
        pass

    # Try to find {"actions": [...]} or {"query": [...]} pattern
    for pattern in [r'\{"actions"\s*:\s*\[.*\]\s*\}', r'\{"query"\s*:\s*\[.*\]\s*\}']:
        m = re.search(pattern, text, re.DOTALL)
        if m:
            try:
                return json.loads(m.group())
            except json.JSONDecodeError:
                pass

    # Try outermost braces/brackets
    for start_char, end_char in [("{", "}"), ("[", "]")]:
        start = text.find(start_char)
        end = text.rfind(end_char)
        if start != -1 and end > start:
            try:
                parsed = json.loads(text[start : end + 1])
                if isinstance(parsed, list):
                    return {"actions": parsed}
                if isinstance(parsed, dict):
                    return parsed
            except json.JSONDecodeError:
                pass

    # Try to extract individual action objects from text
    action_types = ["PlaceRoadLine", "ZoneRect", "PlaceUtility", "PlaceService",
                    "BulldozeRect", "SetTaxRates"]
    actions = []
    for action_type in action_types:
        for m in re.finditer(r'\{"' + action_type + r'"\s*:\s*\{[^}]*\}\s*\}', text):
            try:
                actions.append(json.loads(m.group()))
            except json.JSONDecodeError:
                pass
    if actions:
        return {"actions": actions}

    # Try to fix truncated JSON by adding closing brackets
    for start_char in ["{", "["]:
        start = text.find(start_char)
        if start != -1:
            fragment = text[start:]
            for suffix in ["]}", "]}}", "}", "]}}}", '"]}']:
                try:
                    parsed = json.loads(fragment + suffix)
                    if isinstance(parsed, list):
                        return {"actions": parsed}
                    if isinstance(parsed, dict):
                        return parsed
                except json.JSONDecodeError:
                    continue

    log.warning("Could not parse LLM response: %s", text[:200])
    return {"actions": []}


ZONE_TYPE_ALIASES = {
    "IndustrialLow": "Industrial",
    "IndustrialHigh": "Industrial",
    "OfficeLow": "Office",
    "OfficeHigh": "Office",
    "ResLow": "ResidentialLow",
    "ResHigh": "ResidentialHigh",
    "ComLow": "CommercialLow",
    "ComHigh": "CommercialHigh",
    "Residential": "ResidentialLow",
    "Commercial": "CommercialLow",
    "MixedUse": "MixedUse",
}


def normalize_action(action: dict) -> dict:
    """Fix common LLM mistakes in action parameters."""
    if not isinstance(action, dict):
        return action
    for key, params in action.items():
        if isinstance(params, dict):
            # Fix zone type aliases
            if "zone_type" in params:
                zt = params["zone_type"]
                if zt in ZONE_TYPE_ALIASES:
                    params["zone_type"] = ZONE_TYPE_ALIASES[zt]
            # Fix road type aliases
            if "road_type" in params:
                rt = params["road_type"]
                if rt.lower() == "local":
                    params["road_type"] = "Local"
                elif rt.lower() == "avenue":
                    params["road_type"] = "Avenue"
                elif rt.lower() == "boulevard":
                    params["road_type"] = "Boulevard"
                elif rt.lower() == "highway":
                    params["road_type"] = "Highway"
            # Ensure coordinate values are integers
            for coord_key in ["start", "end", "pos", "min", "max"]:
                if coord_key in params and isinstance(params[coord_key], list):
                    params[coord_key] = [int(round(v)) for v in params[coord_key]]
    return action


def _summarize_action(action: dict) -> str:
    """Create a short summary string for an action."""
    if isinstance(action, dict):
        for key in action:
            return f"{key}({json.dumps(action[key], separators=(',', ':'))})"
    return str(action)[:80]


def play_turn(
    game: GameProcess,
    llm: LLMClient,
    conv_mgr: ConversationManager,
    turn: int,
    ticks_per_turn: int,
) -> tuple:
    """Execute a single game turn with optional query phase."""
    # 1. Step simulation
    game.step(ticks_per_turn)

    # 2. Observe
    obs_response = game.observe()
    observation = obs_response.get("observation", obs_response)

    # 3. Format observation
    user_msg = format_observation(observation, turn)

    # 4. Compute buildable area + water set from overview map on turn 1
    global _buildable_area_info, _water_cells
    if turn == 1 and not _buildable_area_info:
        overview = observation.get("overview_map", "")
        if overview:
            grid_lines = parse_overview_map(overview)
            _buildable_area_info = compute_buildable_area(grid_lines)
            _water_cells = build_water_set(grid_lines)
            if _buildable_area_info:
                log.info("Computed buildable area: %s", _buildable_area_info)
            log.info("Water cells mapped: %d cells", len(_water_cells))

    # 5. Send to LLM
    messages = conv_mgr.build_messages(user_msg)
    response = llm.call(messages)

    # 6. Parse response
    parsed = parse_response(response)

    # 7. If query: fetch layers, send followup
    if "query" in parsed:
        log.info("LLM requested query: %s", parsed["query"])
        try:
            query_response = game.send({
                "cmd": "query",
                "layers": parsed["query"],
            })
            followup_msg = (
                format_layers(query_response)
                + "\n\nNow respond with your actions."
            )
            messages = conv_mgr.build_messages(followup_msg)
            response = llm.call(messages)
            parsed = parse_response(response)
        except Exception as e:
            log.warning("Layer query failed: %s", e)
            parsed = {"actions": []}

    # 8. Execute actions (with normalization + water pre-filtering)
    raw_actions = [normalize_action(a) for a in parsed.get("actions", [])]
    # Pre-filter: skip actions that would land in water or out of bounds
    actions = []
    for a in raw_actions:
        if _action_out_of_bounds(a):
            log.info("  SKIP (bounds): %s", _summarize_action(a))
        elif _action_hits_water(a):
            log.info("  SKIP (water): %s", _summarize_action(a))
        else:
            actions.append(a)

    results = []
    for action in actions:
        try:
            result = game.act(action)
            res_val = result.get("result", "")
            success = res_val == "Success"
            reason = ""
            if not success and isinstance(res_val, dict) and "Error" in res_val:
                reason = res_val["Error"]
                # Track runtime water failures
                if reason == "BlockedByWater":
                    _record_water_failure(action)
            results.append({
                "action": action,
                "result": result,
                "success": success,
                "reason": reason,
                "action_summary": _summarize_action(action),
            })
            if success:
                log.info("  OK: %s", _summarize_action(action))
            else:
                log.warning("  FAIL: %s -> %s", _summarize_action(action), reason or res_val)
        except Exception as e:
            log.error("Action error: %s", e)
            results.append({
                "action": action,
                "result": {"error": str(e)},
                "success": False,
                "reason": str(e),
                "action_summary": _summarize_action(action),
            })

    # 9. Track water failures for directional hints to LLM
    global _consecutive_water_turns, _water_fail_positions
    water_fails_this_turn = [
        r for r in results
        if not r.get("success") and "BlockedByWater" in str(r.get("reason", ""))
    ]
    if water_fails_this_turn:
        _consecutive_water_turns += 1
        for r in water_fails_this_turn:
            act = r.get("action", {})
            for _key, params in act.items():
                if isinstance(params, dict):
                    for coord_key in ["start", "pos", "min"]:
                        if coord_key in params:
                            coords = params[coord_key]
                            if isinstance(coords, list) and len(coords) >= 2:
                                _water_fail_positions.append((coords[0], coords[1]))
                                break
    else:
        _consecutive_water_turns = 0

    # 10. Record turn with metrics
    conv_mgr.record_turn(
        turn, user_msg, response, results,
        treasury=observation.get("treasury", 0),
        population=observation.get("population", {}).get("total", 0),
    )

    return actions, results, observation


def run_session(args: argparse.Namespace):
    api_key = os.environ.get("OPENROUTER_API_KEY")
    if not api_key:
        log.error("OPENROUTER_API_KEY environment variable not set")
        sys.exit(1)

    stats = SessionStats(start_time=time.time())
    llm = LLMClient(model=args.model, api_key=api_key, temperature=args.temperature)
    conv_mgr = ConversationManager(SYSTEM_PROMPT)

    log_dir = Path(args.log_dir)
    log_dir.mkdir(parents=True, exist_ok=True)
    session_id = f"{args.model.split('/')[-1]}_{args.seed}_{int(time.time())}"
    session_log = log_dir / f"{session_id}.jsonl"

    log.info("Session: %s", session_id)
    log.info(
        "Model: %s | Seed: %s | Turns: %d | Ticks/turn: %d",
        args.model, args.seed, args.max_turns, args.ticks_per_turn,
    )

    with GameProcess(args.binary, seed=args.seed) as game:
        if args.seed is not None:
            game.new_game(args.seed)

        for turn in range(1, args.max_turns + 1):
            log.info("--- Turn %d/%d ---", turn, args.max_turns)

            try:
                actions, results, obs = play_turn(
                    game, llm, conv_mgr, turn, args.ticks_per_turn,
                )
            except Exception as e:
                log.error("LLM error on turn %d: %s", turn, e)
                stats.llm_errors += 1
                # Still observe so metrics aren't zeros
                try:
                    obs_resp = game.observe()
                    obs = obs_resp.get("observation", obs_resp)
                except Exception:
                    obs = {}
                actions, results = [], []

            # Track stats
            stats.treasury_history.append(obs.get("treasury", 0))
            pop = obs.get("population", {})
            stats.population_history.append(pop.get("total", 0))
            hap = obs.get("happiness", {})
            stats.happiness_history.append(hap.get("overall", 0))

            # Count actions
            for r in results:
                stats.actions_sent += 1
                if r.get("success"):
                    stats.actions_succeeded += 1
                else:
                    stats.actions_failed += 1
                    log.warning(
                        "Action failed: %s -> %s",
                        r.get("action_summary", "?"),
                        r.get("result", {}),
                    )

            # Always log key metrics
            _pop = pop.get("total", 0)
            _bldgs = obs.get("building_count", 0)
            _attract = obs.get("attractiveness_score", 0)
            _treas = obs.get("treasury", 0)
            _hap = hap.get("overall", 0)
            log.info("  Pop=%d | Bldgs=%d | Happy=%.0f | Attract=%.1f | Treasury=$%.0f", _pop, _bldgs, _hap, _attract, _treas)

            if actions:
                log.info(
                    "Executed %d action(s): %d ok, %d failed",
                    len(actions),
                    sum(1 for r in results if r.get("success")),
                    sum(1 for r in results if not r.get("success")),
                )
            else:
                log.info("No actions this turn")

            # Log turn to JSONL
            turn_log = {
                "turn": turn,
                "tick": obs.get("tick", 0),
                "treasury": obs.get("treasury", 0),
                "population": pop.get("total", 0),
                "happiness": hap.get("overall", 0),
                "power_coverage": obs.get("power_coverage", 0),
                "water_coverage": obs.get("water_coverage", 0),
                "income": obs.get("monthly_income", 0),
                "expenses": obs.get("monthly_expenses", 0),
                "building_count": obs.get("building_count", 0),
                "attractiveness": obs.get("attractiveness_score", 0),
                "warnings": obs.get("warnings", []),
                "actions": actions,
                "results": [
                    {"success": r["success"], "summary": r.get("action_summary", ""),
                     "reason": r.get("reason", "")}
                    for r in results
                ],
            }
            with open(session_log, "a") as f:
                f.write(json.dumps(turn_log, separators=(",", ":")) + "\n")

            stats.turns_played += 1

            # Early exit if bankrupt
            if obs.get("treasury", 0) < -100_000:
                log.warning("City is deeply bankrupt, ending session")
                break

        # Always save replay at session end
        replay_path = str(log_dir / f"{session_id}.replay")
        try:
            game.save_replay(replay_path)
            log.info("Replay saved to %s", replay_path)
        except Exception as e:
            log.warning("Failed to save replay: %s", e)

    # Print summary
    elapsed = time.time() - stats.start_time
    print("\n" + "=" * 60)
    print(f"SESSION COMPLETE: {session_id}")
    print("=" * 60)
    print(f"Model:            {args.model}")
    print(f"Seed:             {args.seed}")
    print(f"Turns played:     {stats.turns_played}")
    print(f"Actions sent:     {stats.actions_sent}")
    print(f"  Succeeded:      {stats.actions_succeeded}")
    print(f"  Failed:         {stats.actions_failed}")
    print(f"LLM errors:       {stats.llm_errors}")
    print(f"Duration:         {elapsed:.0f}s")
    if stats.population_history:
        print(f"Final population: {stats.population_history[-1]}")
    if stats.treasury_history:
        print(f"Final treasury:   ${stats.treasury_history[-1]:,.0f}")
    if stats.happiness_history:
        print(f"Final happiness:  {stats.happiness_history[-1]:.1f}/100")
    print(f"Session log:      {session_log}")
    print("=" * 60)


def main():
    parser = argparse.ArgumentParser(description="Megacity LLM Gameplay Harness")
    parser.add_argument(
        "--binary", default="./target/release/megacity",
        help="Path to game binary (default: ./target/release/megacity)",
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
        "--max-turns", type=int, default=DEFAULT_MAX_TURNS,
        help=f"Max turns to play (default: {DEFAULT_MAX_TURNS})",
    )
    parser.add_argument(
        "--ticks-per-turn", type=int, default=DEFAULT_TICKS_PER_TURN,
        help=f"Simulation ticks per turn (default: {DEFAULT_TICKS_PER_TURN})",
    )
    parser.add_argument(
        "--temperature", type=float, default=0.3,
        help="LLM temperature (default: 0.3)",
    )
    parser.add_argument(
        "--log-dir", default="sessions",
        help="Directory for session logs (default: sessions)",
    )
    parser.add_argument(
        "--save-replay", action="store_true",
        help="(deprecated, replay is always saved)",
    )
    parser.add_argument(
        "-v", "--verbose", action="store_true",
        help="Enable debug logging",
    )
    args = parser.parse_args()

    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)

    run_session(args)


if __name__ == "__main__":
    main()
