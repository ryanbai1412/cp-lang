// CF 1730B Meeting on the Line (float answer): everyone arrives by max(t_i + |x_i - x0|),
// minimised halfway between max(x + t) and min(x - t).
@check fn ok() -> Bool:
    let n = read<1..=1e5>()
    let x = read<[n]0..=1e8>()
    let t = read<[n]0..=1e8>()
    let got: F64 = read_output()
    // the optimum is unique: compare the latest arrival at got with the best possible
    let late = x.keys.map(i => F64(t[i]) + abs(F64(x[i]) - got)).max()!
    let hi = x.keys.map(i => x[i] + t[i]).max()!
    let lo = x.keys.map(i => x[i] - t[i]).min()!
    let best = F64(hi - lo) / 2.0
    if late > best + 1e-6 * max(1.0, best): return false
    true

@!multitest
let n = read<1..=1e5>()
let x = read<[n]0..=1e8>()
let t = read<[n]0..=1e8>()
let hi = x.keys.map(i => x[i] + t[i]).max()!
let lo = x.keys.map(i => x[i] - t[i]).min()!
print(F64(hi + lo) / 2.0)
