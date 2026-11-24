# ioi14p1 review

Score: 3/5
Dream: 2/5

Scored cold against the checklist, the port and the dream (the `## Dream` section of
research/frictions/ioi14p1.md).

`find_location` follows the header: sort by distance from station 0, keep the leftmost C and
rightmost D, classify each station by its turning block. Half the file is the reported judge
stand-in (`get_distance` from `right_d`/`left_c`). Inside `find_location` the stations and
blocks lose their domains.

## Checklist failures

- `find_location(n: Int, first: Block, dist: (Int, Int) -> Int)` and `get_distance(i: Int, j: Int)`
  pass stations as raw `Int` and rebuild them with `S(i)` (lines 24, 74-76) (R6).
- `loc: [n]Int` holds blocks as `Int`, converted back with `Block(loc[s1])`, `Block(loc[a])`
  (lines 25, 36, 53) although `Block` exists (R5, R6). Only the `.ord` difference is reported.

## Owed rewrites

- Declare the station domain before `find_location` and type `loc: [S]Block`, `dist: (S, S) -> Int`,
  `l`, `r: S`; keep `.ord` arithmetic only where a block is computed from distances.

## Language gaps

- A grader fn filling the judge's arrays (E0501) and `.ord` for block differences: in
  research/frictions/ioi14p1.md.

## Dream notes

- Only the grader signature and `...`: the dream never states the algorithm. It should hold
  the classification loop in domain terms (`placed: [Block]?Kind`, `enum Kind = C | D`,
  `l`, `r: Station`), and the port is judged against that.
