// Runnable today/ port of CCC '20 S1 (Surmising a Sprinter's Speed) for proto/cpc.
// Because proto/cpc lacks F64, compares speeds via exact integer cross-multiplication
// and formats the winning ratio rounded to 5 decimal places into a List<Char>.
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
        rev.rev
        for ch in rev: out.push(ch)
    out.push('.')
    var frac = scaled % 100_000
    var div = 10_000
    for _ in 0..5:
        out.push(digit_char(frac / div))
        frac = frac % div
        div = div / 10
    print(out)

main_solve()
