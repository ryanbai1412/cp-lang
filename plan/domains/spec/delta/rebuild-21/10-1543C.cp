// CF 1543C Need for Pink Slips (float answer): recurse over the two non-pink slips; None is
// an invalid slip. Every draw removes at least v >= 0.1, so the recursion is shallow.
@check fn ok() -> Bool:
    let c = read<F64>()
    let m = read<F64>()
    let _p = read<F64>()
    let v = read<F64>()
    // reference: expectation over explicit draw sequences, cash and marker as a pair
    fn go(c: F64, m: F64) -> F64:
        var e = 1.0
        if c > 1e-12:
            let nc = if c <= v + 1e-9: 0.0 else: c - v
            let give = c - nc
            let nm = if m > 1e-12: m + give / 2.0 else: 0.0
            e += c * go(nc, nm)
        if m > 1e-12:
            let nm = if m <= v + 1e-9: 0.0 else: m - v
            let give = m - nm
            let nc = if c > 1e-12: c + give / 2.0 else: 0.0
            e += m * go(nc, nm)
        e
    let want = go(c, m)
    let got: F64 = read_output()
    if abs(got - want) > 1e-6 * max(1.0, abs(want)): return false
    true

@!multitest
let c = read<F64>()
let m = read<F64>()
let _p = read<F64>()
let v = read<F64>()

// a slip of probability a after it is drawn
fn reduced(a: F64) -> ?F64 = if a <= v + 1e-9: None else: a - v
// the other non-pink slip b after a slip of probability a is drawn
fn shared(b: ?F64, a: F64) -> ?F64:
    if let x = b: return x + min(a, v) / 2.0
    None

fn races(c: ?F64, m: ?F64) -> F64:
    var e = 1.0
    if let a = c: e += a * races(reduced(a), shared(m, a))
    if let b = m: e += b * races(shared(c, b), reduced(b))
    e

print(races(c, m), precision: 12)
