# btoi19p5 review

Score: 3/5
Dream: 2/5

Cold review of `port.cp` against the friction's `## Dream`, without the C++. The header derives
the rotation-as-cut argument and the two KMP passes per cut, inside the 4 MB limit. The search
is compact, but positions are raw `Int`s and one copy is not forced.

## Checklist failures
- idiomatic-code/R6: `type Block = (len: Int, in_s: Int, in_t: Int)` (line 16), `0..=s.len`
  and `0..=u.len` (lines 21, 24) and the back-mapping `t.len - back.in_t - back.len` (line 32)
  are all raw positions and cuts.
- idiomatic-code/R10/R3: `List(u.rev)` (line 23) is rebuilt on every cut although `u` never
  changes. The friction blames `prefix_matches` needing an `Indexed` (true, run now in the
  friction: E0606), but the copy could be made once.

## Owed rewrites
- Hoist `List(u.rev)` out of the `cut` loop.

## Language gaps
- `Rev<Str>` is not `Indexed`: in the friction, run added.

## Dream notes
Three lines that call an undefined `longest(u, flip)`. The cut-and-match search and the
flipped start's mapping back to t, the whole problem, are left out. Write the program (send
back).
