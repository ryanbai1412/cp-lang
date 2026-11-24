# mccc6s3

Score: 2/5
Dream: 3/5

Cold review: the sweep and the half-gap search are the editorial, but the nearest-present-value structure is two hand path-halving pointer arrays with sentinel ends, and the friction describes a `DsuOf` port that is no longer there.

## Checklist failures

- R3: `nxt`/`prv` with `above`/`below`/`drop` (lines 37-57) hand-roll path halving; std `DsuOf` exists (`std/dsu.cp`), and an earlier revision (2c753dc70) used it.
- R5: `nxt[m] = m` and `prv[0] = 0` are sentinel ends, `prv` stores "one plus the value" to fit -1, and `best` starts at `Int.MAX` with `moved: -1` for no move (line 21).
- R4: `sweep` runs the scan twice with `back(p)` mirroring and `if not rev:` guards on three lines (68, 70 and in `take`).

## Owed rewrites

- Go back to `DsuOf<U, ?U>` (or a std nearest-present structure), with a measurement if it is slower.

## Language gaps

- The friction says the port uses `DsuOf`; it does not. The hand pointer arrays and their reason are filed in research/frictions/mccc6s3.md.

## Dream notes

- The loop body ends in `...`, and `gap` and `consider` are undefined: the dream never says how a candidate becomes an answer or how the backward sweep works.
