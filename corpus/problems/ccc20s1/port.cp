// Source: CCC 2020 S1 (Surmising a Sprinter's Speed), https://dmoj.ca/problem/ccc20s1
// Problem: n <= 1e5 observations (t, x) of a sprinter, times distinct, |t|, |x| <= 1e9.
//   Print the largest speed he certainly reached (relative error 1e-5).
// Solution: sorted by time, the answer is the largest |dx| / dt between neighbours. Compare
//   the ratios exactly by cross-multiplying (no floats here) and print the winner rounded to
//   5 decimals by integer arithmetic. O(n log n).
// original: corpus/submissions/4323572.cpp

fn digit_char(d: Int) -> Char:
    if d == 0: return '0'
    if d == 1: return '1'
    if d == 2: return '2'
    if d == 3: return '3'
    if d == 4: return '4'
    if d == 5: return '5'
    if d == 6: return '6'
    if d == 7: return '7'
    if d == 8: return '8'
    return '9'

fn main_solve():
    let n: Int = read()
    var obs: List<(Int, Int)> = []
    for _ in 0..n:
        let t: Int = read()
        let x: Int = read()
        obs.push((t, x))
    obs.sort()
    var best_num = 0
    var best_den = 1
    for i in 0..n - 1:
        let a = obs[i]
        let b = obs[i + 1]
        let dx = abs(b.1 - a.1)
        let dt = b.0 - a.0
        if dx * best_den > best_num * dt:
            best_num = dx
            best_den = dt
    let scaled = (best_num * 100_000 * 2 + best_den) / (2 * best_den)
    let whole = scaled / 100_000
    var out: List<Char> = []
    if whole == 0:
        out.push('0')
    else:
        var rev: List<Char> = []
        var w = whole
        while w > 0:
            rev.push(digit_char(w % 10))
            w = w / 10
        for ch in rev.rev: out.push(ch)
    out.push('.')
    var frac = scaled % 100_000
    var div = 10_000
    for _ in 0..5:
        out.push(digit_char(frac / div))
        frac = frac % div
        div = div / 10
    print(out)

main_solve()
