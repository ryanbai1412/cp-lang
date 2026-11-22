# Resume: cold-review lane (CP Lang port and dream scoring)

## Goal and done-bar
Every tracked port in `corpus/problems/<NAME>/` has an independent **port** score and **dream** score
(1-5) in `research/reviews/<NAME>.md`. Each review cites current-source lines, flags syntax that
`decisions.md` rejects (`where` filters, `.pow`, getters called with `()`, comma guards) and hand-rolled
algorithms the stdlib already has, and separates "rewrite today" items from genuine language/stdlib gaps
(which need a `research/frictions/<NAME>.md` report). Bar: `design/EXEMPLARS-PORTS.md`. Under 4/5 goes
back with concrete obligations. 5/5 = no structurally better program is obvious. Without a dream the
port is capped at 3. Don't read `orig.cpp` before scoring. Don't message port authors.

Done-bar: `python3 scripts/review-check.py` shows 0 pending, and every under-4 item is either owed work
listed in its review or a reported gap.

## Current state (measured 2026-11-22, main at 13a9ace1f)
```
python3 scripts/review-check.py
reviews: 305/305 scored, median 4, 18 under 4, 0 pending; new ports 44, 5 under 4; dream-token gap 1.00x
dreams: 305 scored, median 4, 0 sent back
```
The 18 under 4 (all 3/5): agc077c, cf2021d, cf2022e1, cf2039f1, cf2066e, cf2115f1, cf2124f2, cf2129e,
cf2129f2, cf2135f, cf2152h2, cf2180f2, cf2183d2, cf2207c, cf2229e, cf2229i, cf2261g, ucup5s05f.
Most are blocked only on reported stdlib gaps (MeldHeap, BitTrie, Rope/MergeSeg, BucketCount,
xor-potential DSU, persistent segtree, max-plus join/split, complement_components, Cartesian tree).

## What landed (all on main, review lane)
Most recent first: b4b4f1e91 (pairs-spec notes in 3 reviews), 761dd09bf, 4bcbfc6dd, 6250ac6df,
3c3db22ed, c0c7c5eb2, 564e1690a, 3a7a858cc, e9978bef4, 32fa73353, 227d094dc, 614a9f1bb, 3278586d8,
c68fcfca6, 3cf862c3e, b297266a3, 6566cb188, e0f98a0a7, 44b11e499, ca85bc27f, ad6f7d72b, 3cd804828,
53a08f6f1, f7d41dd3c, 9d95454c4, e898f83af, c92915e3e, 318a8b61b, e0e46f58c, fb58aabd5, 804888117,
508ac1967, 43c6c2463, 0e3751a83, 0f7cec442, 0c2c86e90, 240c0809b, 5b2c26988, c27eacfa2, a8c61044b,
ec7b5fe9b, plus earlier `factory: cold-review ...` commits (`git log --format='%h %s' -- research/reviews`).

## In flight
Nothing. No uncommitted review work.

## Next steps (priority order)
1. **Edition switch (decisions.md §212).** `cpl` now defaults to 0.3 (`spec/std`); gates pin
   `CPL_EDITION=0.2`. Under 0.3, 432 of 877 corpus ports fail `cpl check` (under 0.2: 86). All
   existing reviews were checked under 0.2. When a tracked port gets a `spec.cp` (review-check now
   prefers `spec.cp` over `port.cp`; today 100 exist, none in tracked ports), rescore it under 0.3.
   Acceptance: each such port has a review section naming the edition it was checked under.
2. **`.pairs` semantics.** Under 0.3, `xs.pairs` = every `i < j` pair (`xs.adjacent` = neighbours);
   under 0.2 it is still neighbours. Recheck, when they move to 0.3: ceoi22p3 (line 28 wants
   `.adjacent`), cf2178d (lines 22, 26 want `.adjacent`), cf2150f (hand `neighbour_pairs` becomes
   `tree[i].pairs`), abc481g (`events.pairs` as `(prev, cur)` wants `.adjacent`; no review file),
   nccc10s2 (`o.pairs()` looks like it wants all pairs; unconfirmed). Acceptance: sample output
   matches `tests/` for each under 0.3.
3. **Periodic loop.** Every ~15 min: `git pull --ff-only`, `python3 scripts/review-check.py`;
   rescore anything pending in landing order. Acceptance: 0 pending.
4. **Revisit gap-blocked 3s** when the matching stdlib type lands (see list above). Example:
   `Graph.has`/`spanning_tree` are now in `spec/std/graph.cp` (spanning_tree still `todo`) but
   `cpl check` under 0.2 still says E0203; cf2150f's note stays until 0.3.

## How to land
Review-only commits, subject `factory: cold-review N pending ports`, via
`CPL_LANE=review scripts/land.sh <remote>` (serialized train; pre-commit runs
`scripts/check.sh --fast`, ~3-4 min). Then `git fetch` and confirm the SHA on `origin/main`:
the train may rewrite your SHA, so match by subject. No PRs, no `--no-verify`.

## Gotchas and dead ends
- Another lane's `land.sh` runs `git checkout -f` in a shared checkout and erased one batch of
  uncommitted reviews. Check no train is running before editing, and keep copies of in-progress files.
  Don't detect trains with `pgrep -f land.sh` inside a wrapper whose own command line contains it.
- Probe every syntax/API claim with `target/debug/cpl check|run` before writing it. Withdrawn wrong
  advice so far: `.even`, `max(a, b, by:)`, `.drop_first/.drop_last` (all fail); row views
  `dp[k]` of 2-D tables (E0305); passing a named fn to `suffix_scan_in_place` (E0201, needs lambda).
- Spec-vs-compiler drift: `first_some`, `max_subarray`, `{}` empty subset, `Graph.has` are in spec but
  rejected by the 0.2 compiler. Flag as gaps, not as port errors.
- `scout_batch_*` ports aren't counted by review-check.py; scores there don't move totals.
- Dream ratchet: raising `dream_scores` baseline in `research/scoreboard.json` was needed when new
  send-backs were first scores; state it as an assumption each time.
- Fast-gate "gate-fast 297 s FAIL" budget line is a warning, not a failure.

## Open questions for Ryan
1. Should 0.3 be the review bar now? (Most ports would drop below 4 until migrated.)
2. decisions.md §16 says `.step(k)` replaces `by` and `xs.adjacent` replaces `.pairs`, but the 0.2
   compiler accepts both old spellings; should the compiler enforce §16? (Recommended: yes.)
3. cf2066f's dream uses `max_subarray` while the spec says `a.fold(MaxSub)`; which spelling wins?
