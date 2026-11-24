# dmopc21c2p5

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port.

A tree DP by degree: for each node, `free` and `tied` hold polynomial coefficients by component count, merged child by child into a `[Deg]List<Z>` table.

## Checklist failures

- `ways` and `next` are `[Deg]List<Z>` tables sized by hand (`[Z(0) for _ in 0..size]`) and merged with nested index loops; a polynomial product would state the merge.
- `fact` is filled by a loop over `Deg` with `if let up = d.next`, where a `Comb` or a prefix product exists.
- `for i in 0..=seen if ways[d][i] != 0` skips zero coefficients by testing `Mod` values, a C++ optimisation.
- `MOD = 998_244_352` is one less than the usual prime and has no comment saying why.
