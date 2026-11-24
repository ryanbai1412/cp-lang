"""The spec and conformance gates behind `scripts/spec-gate.py`.

Every gate is a module here with `HELP`, `args(parser)` and
`run(args) -> list[Problem]`; the entry prints every problem as
`spec-gate <gate>: <where>: <what>` and a one-line verdict, and exits 1 on
any problem.
"""
from __future__ import annotations

import dataclasses
import pathlib

ROOT = pathlib.Path(__file__).resolve().parents[2]
CONFORMANCE = ROOT / "spec" / "conformance"


@dataclasses.dataclass(frozen=True)
class Problem:
    where: str
    what: str


def failing() -> dict[str, str]:
    """case id -> verdict for every conformance case research/scoreboard.json
    records as not passing: the one record of a known failure."""
    import json
    board = ROOT / "research" / "scoreboard.json"
    if not board.is_file():
        return {}
    cases = json.loads(board.read_text()).get("suites", {}).get("conformance", {}).get("cases", {})
    return {c: v for c, v in sorted(cases.items()) if v != "pass"}


def cases() -> list[pathlib.Path]:
    return sorted(CONFORMANCE.rglob("*.cp"))


def read(path: pathlib.Path) -> str:
    """The case's text with every byte kept (a lone `\r` is case data)."""
    return path.read_bytes().decode()


def write(path: pathlib.Path, text: str) -> None:
    path.write_bytes(text.encode())


def case_id(path: pathlib.Path) -> str:
    return path.relative_to(CONFORMANCE).as_posix()


def header(text: str) -> list[str]:
    """The leading `// ...` lines of a case."""
    out = []
    for line in text.split("\n"):
        if not line.lstrip().startswith("//"):
            break
        out.append(line.strip())
    return out


def header_values(text: str, key: str) -> list[str]:
    tag = key + ":"
    return [l[2:].strip()[len(tag):].strip() for l in header(text) if l[2:].strip().startswith(tag)]
