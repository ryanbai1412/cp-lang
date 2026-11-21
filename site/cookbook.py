"""Cookbook generator backed directly by validated ports in `proto/ports/ecnerwala/`.

Every recipe reads its live `.cp` source from disk at build time so cookbook
code never drifts from the compiler or corpus gate. `site/check_docs.py`
verifies every referenced port and runs its sample cases.
"""

from __future__ import annotations

import html
from pathlib import Path
import re

COOKBOOK_CATEGORIES = [
    (
        "Trees & graphs",
        "trees-and-graphs",
        [
            ("cf2183d1", "Tree depth bucketing with `Rooted`", "Root a tree with `Rooted(n, edges)` and bucket vertices by depth in 18 lines."),
            ("cf2207d", "Bottom-up tree DP on `Rooted`", "Compute minimum leaf distances over a rooted tree using post-order traversal."),
            ("cf2245e", "Disjoint set union (`Dsu`) on trees", "Merge components with `Dsu<V>` and typed domain indices."),
            ("ucup5s06g", "Cycle & reachability graph", "Identify vertices on or below directed cycles using `std.graph`."),
            ("ucup4s25c", "Directed graph construction & `std.seq`", "Build an ordered adjacency graph from sequence constraints."),
        ],
    ),
    (
        "Range queries & data structures",
        "range-queries",
        [
            ("cf2201d", "Segment tree (`SegTree`) with custom monoid", "Maintain per-value `[first, last]` spans in a point-update segment tree."),
            ("cf2147f", "Lazy segment tree (`LazySegTree`)", "Handle range updates and range queries with a composable lazy action."),
            ("cf2101b", "Fenwick tree (`Fenwick`) & inversion parity", "Count parity of inversions in O(n log n) with a Fenwick tree."),
            ("ucup4s24f", "Priority queue (`Heap`) over prefix sums", "Scan prefix sums right-to-left while maintaining candidates in a `Heap`."),
        ],
    ),
    (
        "Dynamic programming",
        "dynamic-programming",
        [
            ("cf2069c", "Modular counting DP (`Mod<P>`)", "Count valid subsequences modulo 998244353 with zero manual `% MOD` boilerplate."),
            ("cf2255b", "Combinatorial block DP (`Mod<P>` + `Comb`)", "Count sequences with fixed maximal blocks using `Mod` and `Comb`."),
            ("cf2268c", "2D grid & subarray threshold DP", "Index a 2D table with distinct row/column domains and zero bounds checks."),
            ("cf2147h", "Bitset & max-flow coloring (`Bitset`)", "Combine `Bitset` operations with graph coloring on dense constraints."),
        ],
    ),
    (
        "Math & combinatorics",
        "math-and-combinatorics",
        [
            ("cf2211b", "Divisor counting with `Mod<P>`", "Count divisors modulo a prime in 15 lines."),
            ("cf2201f1", "Ferrers diagram transposition (`std.math`)", "Verify transposed partition diagrams directly over domain ranges."),
            ("cf2147g", "Modular inverses & factorials (`std.math`)", "Compute modular products and inverses cleanly with `Mod`."),
        ],
    ),
    (
        "Binary search, greedy & interactive",
        "greedy-and-interactive",
        [
            ("cf2165a", "Cyclic adjacent merging (`std.seq`)", "Express a cyclic greedy reduction in 8 lines."),
            ("cf2147e", "Binary search on answer", "Use `first_true` / `last_true` without off-by-one boundary bugs."),
            ("cf2178e", "Interactive binary search (`@!interactive` + `ask`)", "Halve a range via interactive prefix-sum queries with automatic flushing."),
            ("cf2135b", "Interactive 2D coordinate probing", "Locate a hidden point with directional `ask(...)` queries."),
        ],
    ),
    (
        "Strings & geometry",
        "strings-and-geometry",
        [
            ("ucup5s05f", "String algorithms & combinatorics (`std.strings`)", "Combine suffix/string processing with modular binomial sums."),
            ("ucup5s05e", "Computational geometry (`std.geometry`)", "Answer 2D point containment queries using `std.geometry` primitives."),
        ],
    ),
]


def all_recipes() -> list[tuple[str, str, str, str]]:
    """Return `(category_title, port_name, title, summary)` for every recipe."""
    out = []
    for cat_title, _, items in COOKBOOK_CATEGORIES:
        for port_name, title, summary in items:
            out.append((cat_title, port_name, title, summary))
    return out


def port_path(repo_root: Path, port_name: str) -> Path:
    return repo_root / "proto/ports/ecnerwala" / f"{port_name}.cp"


def render_cookbook_html(repo_root: Path, highlight_cp) -> str:
    """Render the full body HTML for `cookbook.html`."""
    total_ports = len(list((repo_root / "proto/ports/ecnerwala").glob("*.cp")))
    recipes = all_recipes()
    parts = [
        f"<p>Every recipe below is a complete, real contest solution pulled directly from the "
        f"validated corpus (<code>proto/ports/ecnerwala/</code>, currently {total_ports} ports). "
        f"All {len(recipes)} recipes are checked, formatted, linted, compiled to C++17 under 64 KB, "
        f"and sample-tested in <code>scripts/check.sh --fast</code>.</p>",
    ]
    for cat_title, cat_slug, items in COOKBOOK_CATEGORIES:
        parts.append(f'<h2 id="{cat_slug}">{html.escape(cat_title)}</h2>')
        for port_name, title, summary in items:
            src_path = port_path(repo_root, port_name)
            source = src_path.read_text()
            line_count = len(source.splitlines())
            recipe_id = f"recipe-{port_name}"
            parts.append(f'<h3 id="{recipe_id}">{html.escape(title)}</h3>')
            parts.append(
                f"<p>{html.escape(summary)} "
                f'<small>(<code>proto/ports/ecnerwala/{html.escape(port_name)}.cp</code>, '
                f"{line_count} lines)</small></p>"
            )
            parts.append(
                '<div class="code-block">'
                f'<div class="code-bar"><span>{html.escape(port_name)}.cp</span>'
                '<button class="copy" type="button" data-copy>Copy</button></div>'
                f'<pre><code class="language-cp">{highlight_cp(source)}</code></pre>'
                "</div>"
            )
    return "\n".join(parts)
