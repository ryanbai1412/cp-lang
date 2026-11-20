#!/usr/bin/env python3
"""Embed a JSON capture in the offline HTML template. No third-party packages."""

import argparse
import json
from pathlib import Path
import re

DIRECTORY = Path(__file__).resolve().parent


def safe_json(value):
    """JSON inside a script element must never contain an HTML end tag."""
    return (json.dumps(value, ensure_ascii=False, separators=(",", ":"), allow_nan=False)
            .replace("<", "\\u003c").replace("\u2028", "\\u2028").replace("\u2029", "\\u2029"))


def embed_payload(html, element_id, value):
    pattern = rf'(<script id="{re.escape(element_id)}" type="application/json">)[\s\S]*?(</script>)'
    result, count = re.subn(pattern, lambda m: m[1] + safe_json(value) + m[2], html)
    if count != 1:
        raise ValueError(f"Template must contain exactly one {element_id} payload")
    return result


def render_html(trace, template=None):
    """Compiler integration seam; input is one trace-schema.json document.

    Full semantic validation (types, source lines, graph endpoints, checkpoints)
    also runs in the viewer, so malformed captures show an actionable error.
    """
    if not isinstance(trace, dict) or trace.get("schemaVersion") != 1:
        raise ValueError("Expected a CP Lang trace with schemaVersion: 1")
    html = template if template is not None else (DIRECTORY / "index.html").read_text()
    html = embed_payload(html, "trace-data", trace)
    return embed_payload(html, "trace-samples", [])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("trace", type=Path, help="JSON execution capture")
    parser.add_argument("output", type=Path, help="standalone HTML destination")
    args = parser.parse_args()
    try:
        # Reject JavaScript's non-JSON NaN/Infinity before producing an artifact.
        trace = json.loads(args.trace.read_text(), parse_constant=lambda x: (_ for _ in ()).throw(ValueError(f"Invalid JSON constant: {x}")))
        result = render_html(trace)
    except (OSError, ValueError) as error:
        parser.exit(1, f"Cannot emit trace: {error}\n")
    args.output.write_text(result)
    print(f"Trace: {args.output}")


if __name__ == "__main__":
    main()
