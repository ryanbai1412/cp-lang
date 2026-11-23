// Codeforces 2165 D — Path Split
//
// Partition `vals[0..n]` into the fewest subsequences whose consecutive
// elements differ by +-1. Minimizing chains equals `n` minus the maximum
// number of directed transitions `i -> j` (`i < j`, `|vals[i] - vals[j]| == 1`)
// with each index having in-degree <= 1 and out-degree <= 1.
//
// Every transition links an even value to an adjacent odd value, and the two
// degree bounds decouple into two independent matchings:
// - `Flip.Even`: edges entering odd values (`even < odd`) and edges leaving
//   odd values (`odd < even`). Reflecting every even value's positions via
//   `reflect(xs)` reverses their order so both directions become the uniform
//   condition `carried_pos < pos` once `unmatched_prev` is reflected into
//   `v`'s coordinate frame at each step.
// - `Flip.Odd`: symmetrically, edges entering and leaving even values.
//
// Sweep `v = 0..=2n` with `carried` holding the unmatched positions of `v - 1`
// in sorted order (already reflected into `v`'s coordinate frame). As `pos`
// increases through `positions`, `ptr` advances monotonically over `carried`
// and `available` counts carried positions strictly before `pos`. Every
// available earlier position can match `pos` or any later position equally
// well, so greedily consuming one whenever `available > 0` is optimal by an
// exchange argument; unmatched positions of `v` stay in `unmatched` for `v + 1`.

enum Flip:
    Even
    Odd

@!multitest
let n: 1..=1_000_000 = read()
let vals: [0..n]1..=2 * n = read()
var by_val: [0..=2 * n]List<Int> = []
for (i, v) in vals.items: by_val[v].push(i)

fn reflect(xs: List<Int>) -> List<Int> = [n - 1 - x for x in xs.rev]

var ans = n
for flip in [Flip.Even, Flip.Odd]:
    var unmatched_prev: List<Int> = []
    for v in 0..=2 * n:
        let is_flipped = match flip:
            Flip.Even => v % 2 == 0
            Flip.Odd => v % 2 == 1
        let positions = if is_flipped: reflect(by_val[v]) else: by_val[v]
        let carried = reflect(unmatched_prev)
        var (ptr, available) = (0, 0)
        var unmatched: List<Int> = []
        for pos in positions:
            while ptr < carried.len and carried[ptr] < pos:
                ptr += 1
                available += 1
            if available > 0:
                available -= 1
                ans -= 1
            else:
                unmatched.push(pos)
        unmatched_prev = unmatched
print(ans)
