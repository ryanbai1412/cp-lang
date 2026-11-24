// CCC 2000 S5, tuned: port.cp's half-line cuts compared as raw fractions with a positive
// denominator by cross-multiplying, instead of building a reduced `Ratio` (one gcd) per pair.
const SCALE = 1000

let n: 1..=1000 = read()
let read_at: [n](x: F64, y: F64) = read()
let sheep = [(x: round(s.x * SCALE), y: round(s.y * SCALE)) for s in read_at]

for p in sheep:
    var (lo_p, lo_q) = (0, 1)
    var (hi_p, hi_q) = (1000 * SCALE, 1)
    var open = true
    for q in sheep:
        let gap = q.x * q.x + q.y * q.y - p.x * p.x - p.y * p.y
        if q.x == p.x:
            if q.y < p.y:
                open = false
                break
        elif q.x < p.x:
            if -gap * lo_q > lo_p * 2 * (p.x - q.x): (lo_p, lo_q) = (-gap, 2 * (p.x - q.x))
        elif gap * hi_q < hi_p * 2 * (q.x - p.x): (hi_p, hi_q) = (gap, 2 * (q.x - p.x))
    if open and lo_p * hi_q <= hi_p * lo_q:
        print("The sheep at ({fixed(p.x, SCALE, 2)}, {fixed(p.y, SCALE, 2)}) might be eaten.")
