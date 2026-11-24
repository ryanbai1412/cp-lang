# ioi25_migrations

Score: 2/5
Dream: 3/5

Scored cold on main against the statement (IOI 2025 Day 2, Migrations), `dream.cp` and
`port.cp`. The dream is under 4, so it goes back and the port score is provisional.

## Dream failures

- The dream is the port (`dream.cp:11-126` is `port.cp:18-130` with `@grader(two_phase)`).
- `dream.cp:83-85`, `say` assigns the global `said` and returns it, and lines 107-111 call it
  for both: a side effect in an expression (idiomatic-code/R7, caps the dream at 3). Rewrite:
  the sender's state as one `struct Sender` (tree, plan, lists, ends, last symbol) owned by
  `send_message`, and `say` returning the symbol that the caller stores.
- `dream.cp:74-81`: eleven top-level `var`s carry the first run's state, reset by hand at
  `at == 1` (lines 89-91) (R7: no state carried from one test to the next).
- `dream.cp:13` declares `domain Site`, but candidates, ends and blocks are `List<Int>` and
  `Int` everywhere except `tree` (R6).
- `dream.cp:107-109`, `p.resized(4, fill: 0)` pads with site 0, a real site (R5). Use
  `?Site` or say in the header why 0 is safe as filler.
- `dream.cp:41-64`: the case tables are the editorial's, but the header doesn't say what a
  symbol means at each of the last three sites, so the tables can't be checked against it (R1).

## Checklist failures (port)

- No `research/frictions/ioi25_migrations.md` (porting step 4): the state, the `Site` domain
  and the grader split have no report, and there is no time/RSS measurement against orig.cpp
  at `--max` (R10). Route: the author.
- The port has every dream failure above (`port.cp:81-92, 94-117`).
- No `// limits:` line in the header (lines 1-17).

## Fixed in this review

- `cpl lint` warned 14 times on `port.cp`; 13 are fixed, with no change to the algorithm:
  - `phase_plan`'s local `plan` is now `phases` (line 29).
  - `heard` and `say` use `find`, not `first` (L0509, lines 44 and 91).
  - `longest_path`'s locals no longer hide the sender's globals (`sites`, `ps`/`qs`, `ps2`/`qs3`,
    lines 120-130).
  - The driver's `(a, b)` is now `(end1, end2)` (line 138).
  
  `scripts/corpus.py verify ioi25_migrations` stays clean.
- Left: L0507 on line 135 wants `[0] + [send_message(...) for i in 1..n]`. `send_message`
  changes the sender's state on every call, so a comprehension would hide that (R7). This is
  a lint false positive for stateful calls. Route: lint.

## Port nits from the friction triage

- `dream.cp:117`, `let s = [Int(x) for x in s]` is not needed: a `List<0..=4>` already passes
  where `heard` takes a `List<Int>` (a three-line repro prints `3`), as `port.cp:43` does.
- `List<Site>` and `?Site` filler compile today (`var p: List<Site> = [Site(0)]`, then
  `p.map(x => Some(x)).resized(8, fill: None)`), so the `List<Int>` candidates and the
  `resized(4, fill: 0)` filler above are rewrites, not language gaps.
