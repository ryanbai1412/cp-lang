# Friction report: egoi23p7

`port.cp` is now the body of `dream.cp`, which compiled and was verified. The v0.2 port it replaces was the accepted C++ translated:
- recursive `dfs_req`/`dfs_comp` over global `vis`, `has_cyc` and `ord`;
- component ids as raw `Int` with `comps: List<List<V>>`;
- requirements in parallel `req_u`/`req_v` lists;
- a `Set<Comp>` of unvisited trees rebuilt from `to_vis` at every visit, which raised four L0412 lint warnings.

The rewrite takes the dream's spelling for each of these:
- the forest is `Rooted.try_from_parents(up) else: print("NO")`;
- blocks are the forest's roots, and the bans are tallied by `forest.root_of`;
- the complement search is a `todo` stack over the unreached `left` list.

The header's `Solution:` now describes that program.

## Dream

`dream.cp` is the ideal and the port is identical to it, so there is no dream/port gap.

## Banned parents are kept twice to drop duplicate input pairs
- **Where:** lines 21-26: `var bans: [V]Set<V>` and `var banned_by: [V]List<V>`, pushing to the list only when the set insert is new.
- **What felt wrong:** "the distinct banned parents of c" is one idea, stored as a membership set plus an insertion-order list.
- **Goal it hurts:** express.
- **Ideal code:** `let bans = read<[m2](child: V, parent: V)>().group(by: e => e.child, to: e => e.parent).distinct()` as `[V]Set<V>`, iterated directly by `tally`.
- **General fix:** iterating the `Set` would do today but costs a tree walk per block. A grouped-read helper (`group(by:, to:)` into a `[K]`-indexed set) would state it. Owner: stdlib.
- **Severity:** low.

## Measurements

- **Tokens:** port 575, accepted C++ 1343 (comments excluded).
- **Runtime and memory on gen.py --max:** port 1.07 s / 78 MB, accepted C++ 0.72 s / 96 MB.
- **Emitted C++:** 32651 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
