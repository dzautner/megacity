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
DEFAULT_TICKS_PER_TURN = 100
DEFAULT_MAX_TURNS = 200
PROTOCOL_VERSION = 1

SYSTEM_PROMPT = """\
## You Are Playing Megacity

You are a city builder AI managing a 256x256 grid city. Each turn (~1 game day),
you receive the city state and respond with building actions. Your goal: maximize
population, happiness, and treasury.

## Coordinate System
- Grid: 256x256 cells. (0,0) = top-left, (255,255) = bottom-right
- All actions use (x, y) grid coordinates as integers
- Water cells (~) cannot be built on
- The overview map shows 64x64 (each char = 4x4 cells)
- The detail map shows 1:1 (each char = 1 cell, coordinates labeled)

## Game Mechanics

### Roads -- BUILD THESE FIRST
| Type      | Cost | Maintenance | Speed | Allows Zoning |
|-----------|------|-------------|-------|---------------|
| Local     | $10  | $0.3/mo     | 30    | Yes           |
| Avenue    | $20  | $0.5/mo     | 50    | Yes           |
| Boulevard | $30  | $1.5/mo     | 60    | Yes           |
| Highway   | $40  | $2.0/mo     | 100   | NO            |

PlaceRoadLine draws a straight line between two grid points.
Roads carry power and water from utilities to buildings via the network.

### Zoning -- Place ADJACENT to Roads
| Zone Type       | Max Level | L1 Cap | L5 Cap  |
|-----------------|-----------|--------|---------|
| ResidentialLow  | 3         | 10     | 80      |
| ResidentialHigh | 5         | 50     | 2000    |
| CommercialLow   | 3         | 8      | 60      |
| CommercialHigh  | 5         | 30     | 1200    |
| Industrial      | 5         | 20     | 600     |
| Office          | 5         | 30     | 1500    |

ZoneRect sets a rectangle of cells to a zone type. Only grass cells adjacent
to a road will be zoned. Buildings spawn automatically in zoned cells that
have road access + power + water.

### Utilities -- Build BEFORE Zoning
| Type        | Cost   | Range | Notes                    |
|-------------|--------|-------|--------------------------|
| PowerPlant  | $1000  | 30    | Covers ~30 cells via roads |
| WaterTower  | $200   | 25    | Covers ~25 cells via roads |
| SolarFarm   | $1200  | 20    | Lower output, clean      |
| WindTurbine | $600   | 15    | Weather dependent        |

Power and water propagate through the road network from utility positions.
Buildings without both power AND water will not grow.

### Services -- Build as Population Grows
| Type             | Cost   | Radius | Monthly |
|------------------|--------|--------|---------|
| FireStation      | $800   | 16     | ~$100   |
| PoliceStation    | $600   | 16     | ~$80    |
| Hospital         | $2000  | 25     | ~$200   |
| ElementarySchool | $2000  | 20     | ~$150   |
| HighSchool       | $1500  | 20     | ~$120   |

Services cover buildings within their radius. Gaps = unhappy citizens.

### Economy
- Income: property tax on buildings. Higher level buildings = more tax.
- Expenses: road maintenance + service maintenance + utility upkeep
- Starting treasury: $50,000
- Default tax rate: 9% per zone type (adjustable 0-100%)
- Bankruptcy (deep negative treasury) = game over

### Zone Demand (0-100 scale)
- Residential demand: driven by jobs availability
- Commercial demand: driven by population needing shops
- Industrial demand: driven by goods demand
- High demand = buildings upgrade faster, more immigrants
- Balance all four for healthy growth

### Happiness (0-100)
- Affected by: employment, services, commute time, pollution, taxes, housing
- High happiness -> immigration -> population growth
- Low happiness -> emigration -> population decline

## Available Actions
```json
{"PlaceRoadLine": {"start": [x,y], "end": [x,y], "road_type": "Local"|"Avenue"|"Boulevard"|"Highway"}}
{"ZoneRect": {"min": [x,y], "max": [x,y], "zone_type": "ResidentialLow"|"ResidentialHigh"|"CommercialLow"|"CommercialHigh"|"Industrial"|"Office"}}
{"PlaceUtility": {"pos": [x,y], "utility_type": "PowerPlant"|"WaterTower"|"SolarFarm"|"WindTurbine"}}
{"PlaceService": {"pos": [x,y], "service_type": "FireStation"|"PoliceStation"|"Hospital"|"ElementarySchool"|"HighSchool"}}
{"BulldozeRect": {"min": [x,y], "max": [x,y]}}
{"SetTaxRates": {"residential": 0.09, "commercial": 0.09, "industrial": 0.09, "office": 0.09}}
```

## Querying Layers
Before acting, you may request detail layers:
```json
{"query": ["map", "buildings", "services", "utilities", "roads", "zones", "terrain"]}
```
You'll receive the data, then respond with your actions.

## Strategy Guide
Turn 1-3: Build a main avenue through the center. Place PowerPlant and WaterTower
          near it. Zone ResidentialLow and CommercialLow along the avenue.
Turn 4-10: Extend road network with local roads. Expand zones. Add FireStation.
Turn 10-20: Add PoliceStation, first school. Zone Industrial (away from residential).
Turn 20+: Add Hospital. Expand to ResidentialHigh/CommercialHigh. Balance budget.

## Response Format
Respond with ONLY a JSON object:
- To query: {"query": ["map", "buildings"]}
- To act: {"actions": [{"PlaceRoadLine": {...}}, ...]}
- To pass: {"actions": []}
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

    def record_turn(self, turn: int, observation: str, response: str, results: list):
        """Record a completed turn and compress history periodically."""
        self.recent_turns.append((observation, response))
        if len(self.recent_turns) > 6:
            self.recent_turns.pop(0)
        self.turn_log.append({
            "turn": turn,
            "obs_snippet": observation[:200],
            "results": results,
        })
        if turn > 0 and turn % 10 == 0:
            self._compress_history(turn)

    def _compress_history(self, current_turn: int):
        """Summarize the last 10 turns into a compact history block."""
        block_start = max(1, current_turn - 9)
        block = self.turn_log[block_start - 1 : current_turn]
        actions_taken = sum(len(t.get("results", [])) for t in block)
        summary = f"Turns {block_start}-{current_turn}: {actions_taken} actions executed."
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
            "max_tokens": 2048,
        }

        for attempt in range(3):
            try:
                resp = requests.post(
                    OPENROUTER_URL, headers=headers, json=payload, timeout=60,
                )
                resp.raise_for_status()
                return resp.json()
            except (requests.RequestException, json.JSONDecodeError) as e:
                log.warning("API attempt %d failed: %s", attempt + 1, e)
                if attempt < 2:
                    time.sleep(2 ** attempt)
        raise RuntimeError("OpenRouter API failed after 3 attempts")


def format_observation(obs: dict) -> str:
    """Format a city observation into a rich text summary for the LLM."""
    parts = [f"## Turn Observation (tick {obs.get('tick', '?')})"]

    # Economy
    treasury = obs.get("treasury", 0)
    income = obs.get("monthly_income", 0)
    expenses = obs.get("monthly_expenses", 0)
    net = obs.get("net_income", income - expenses)
    parts.append("\n### Economy")
    parts.append(f"Treasury: ${treasury:,.0f}")
    parts.append(f"Monthly income: ${income:,.0f}")
    parts.append(f"Monthly expenses: ${expenses:,.0f}")
    parts.append(f"Net: ${net:,.0f}")

    # Population
    pop = obs.get("population", {})
    total = pop.get("total", 0)
    employed = pop.get("employed", 0)
    unemployed = pop.get("unemployed", 0)
    homeless = pop.get("homeless", 0)
    parts.append("\n### Population")
    parts.append(f"Total: {total}")
    parts.append(f"Employed: {employed} | Unemployed: {unemployed} | Homeless: {homeless}")

    # Zone demand
    zd = obs.get("zone_demand", {})
    parts.append("\n### Zone Demand (0-100)")
    parts.append(
        f"R: {zd.get('residential', 0):.0f} | "
        f"C: {zd.get('commercial', 0):.0f} | "
        f"I: {zd.get('industrial', 0):.0f} | "
        f"O: {zd.get('office', 0):.0f}"
    )

    # Coverage
    power_cov = obs.get("power_coverage", 0)
    water_cov = obs.get("water_coverage", 0)
    svcs = obs.get("services", {})
    parts.append("\n### Coverage")
    parts.append(f"Power: {power_cov:.0%} | Water: {water_cov:.0%}")
    parts.append(
        f"Fire: {svcs.get('fire', 0):.0%} | "
        f"Police: {svcs.get('police', 0):.0%} | "
        f"Health: {svcs.get('health', 0):.0%} | "
        f"Education: {svcs.get('education', 0):.0%}"
    )

    # Happiness
    hap = obs.get("happiness", {})
    overall = hap.get("overall", 0)
    parts.append(f"\n### Happiness: {overall:.1f}/100")

    # Warnings
    warnings = obs.get("warnings", [])
    if warnings:
        parts.append("\n### Warnings")
        for w in warnings:
            parts.append(f"  - {w}")

    # Recent action results
    recent_results = obs.get("recent_action_results", [])
    if recent_results:
        parts.append("\n### Recent Action Results")
        for r in recent_results[-8:]:
            status = "OK" if r.get("success") else "FAIL"
            reason = r.get("reason", "")
            summary = r.get("action_summary", "?")
            line = f"  [{status}] {summary}"
            if not r.get("success") and reason:
                line += f" -- {reason}"
            parts.append(line)

    # Overview map
    overview = obs.get("overview_map", "")
    if overview:
        parts.append("\n### Overview Map (64x64, each char = 4x4 cells)")
        parts.append(overview)

    parts.append(
        "\nRespond with a JSON object. To query layers: "
        '{"query": ["map", ...]}. To act: {"actions": [...]}'
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
    text = content.strip()
    # Strip markdown fences
    if text.startswith("```"):
        lines = text.split("\n")
        lines = [l for l in lines if not l.strip().startswith("```")]
        text = "\n".join(lines).strip()

    try:
        parsed = json.loads(text)
        if isinstance(parsed, dict):
            return parsed
        if isinstance(parsed, list):
            return {"actions": parsed}
    except json.JSONDecodeError:
        pass

    # Try to find JSON in text
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

    log.warning("Could not parse LLM response: %s", text[:200])
    return {"actions": []}


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
    user_msg = format_observation(observation)

    # 4. On turn 1, auto-query all layers for spatial awareness
    if turn == 1:
        try:
            query_response = game.send({
                "cmd": "query",
                "layers": ["map", "buildings", "services", "utilities", "terrain"],
            })
            user_msg += "\n\n" + format_layers(query_response)
        except Exception as e:
            log.warning("Initial layer query failed: %s", e)

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

    # 8. Execute actions
    actions = parsed.get("actions", [])
    results = []
    for action in actions:
        try:
            result = game.act(action)
            success = result.get("result") == "Success"
            results.append({
                "action": action,
                "result": result,
                "success": success,
                "action_summary": _summarize_action(action),
            })
        except Exception as e:
            log.error("Action error: %s", e)
            results.append({
                "action": action,
                "result": {"error": str(e)},
                "success": False,
                "action_summary": _summarize_action(action),
            })

    # 9. Record turn
    conv_mgr.record_turn(turn, user_msg, response, results)

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
                actions, results, obs = [], [], {}

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
                "warnings": obs.get("warnings", []),
                "actions": actions,
                "results": [
                    {"success": r["success"], "summary": r.get("action_summary", "")}
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
        "--max-turns", type=int, default=DEFAULT_MAX_TURNS,
        help=f"Max turns to play (default: {DEFAULT_MAX_TURNS})",
    )
    parser.add_argument(
        "--ticks-per-turn", type=int, default=DEFAULT_TICKS_PER_TURN,
        help=f"Simulation ticks per turn (default: {DEFAULT_TICKS_PER_TURN})",
    )
    parser.add_argument(
        "--temperature", type=float, default=0.7,
        help="LLM temperature (default: 0.7)",
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
