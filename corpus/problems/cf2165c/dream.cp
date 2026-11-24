// Codeforces 2165 C — Binary Wine
//
// All values `a[i]` and query targets `c` are `< 2^BITS` (`BITS = 30`). We
// match bits `b` of `c` from MSB (`BITS - 1`) down to `0`:
// - If `cnt > want` (a bit of `c` is `0` while some element has bit `b` set, or
//   two elements already have bit `b` set), the extra `1` at bit `b` dominates
//   all lower bits `0..b`, so no further increments are needed (`break`).
// - If `cnt < want` (`c` wants bit `b` set and no element has it), we must
//   raise one element to the next multiple of `2^b`, costing `(1 << b) - (x & mask)`.
//   Picking the element maximizing `x & mask` minimizes both this step's cost
//   and leaves it at `0` on all lower bits (identical for future steps).
// Because at most `BITS` elements are ever incremented and larger elements
// dominate on high bits, only the `MAX_USED = min(n, BITS)` largest elements
// of `a` can ever participate.

use std.seq.{argmax}

const BITS = 30

@!multitest
let (n, q): (1..=500_000, 1..=50_000) = read()
let a: [0..n]0..=2 ** 30 - 1 = read()
let top = a.sorted(by: x => desc(x))[..min(n, BITS)].to_list()

for _ in 0..q:
    var buf = top
    let c: 0..=2 ** 30 - 1 = read()
    var ans = 0
    for b in (0..BITS).rev:
        let want = Int(c.bit(b))
        let cnt = count(x.bit(b) for x in buf)
        if cnt > want: break
        if cnt < want:
            let mask = (1 << b) - 1
            let best_i = buf.argmax(by: x => x & mask)!
            let add = (1 << b) - (buf[best_i] & mask)
            ans += add
            buf[best_i] += add
    print(ans)
