// Source: NOI 2008 P5 (Candy Rain), https://dmoj.ca/problem/noi08p5
// Problem: n <= 2e5 events with nondecreasing times 0 <= T <= 2e9 on a sky [0, len], len <= 1000:
//   insert a cloud of colour 1 <= c <= 1e6 on [l, r] (0 <= l <= r <= len) moving left (d = -1)
//   or right (d = 1), its left end bouncing off 0 and len (one cloud per colour at a time);
//   delete the cloud of a colour that is in the sky; or print how many colours a bag [L, R]
//   (0 <= L <= R <= len) catches at time T, i.e. how many clouds overlap it.
// Solution: a cloud's left end is periodic with p = 2 len: as a phase x in [0, p), x < len
//   moves right and x >= len is the mirrored leftward leg. Store each cloud, by its phase at
//   time 0 and width w, as a point in one of two sheared planes, (x, x + w) and
//   (x + p, x - w + p); a query shifts by -T and counts one rectangle per plane, plus its copy
//   shifted by p for wraparound, in a p by (p + len) 2D Fenwick tree. The planes are counted in
//   two passes over the stored events so one tree fits the memory limit; a delete finds its cloud
//   by colour in an array. O(n log^2 len).
// original: corpus/submissions/6067573.cpp

let (n, len): (1..=200_000, 1..=1000) = read()
let p = 2 * len
// A plane's points have x in [ox, ox + p) and y in [oy, oy + p + len); (ox, oy) is (0, 0) for
// the rightward plane and (p, p - len) for the leftward one.
domain PX = 0..p
domain PY = 0..p + len
domain Colour = 1..=1_000_000

// Cloud(x, w, v): a cloud of phase x at time 0 and width w enters (v = 1) or leaves (v = -1).
enum Event = Cloud(x: Int, w: Int, v: -1..=1) | Ask(t: Int, l: Int, r: Int)

// phase[c], width[c]: the cloud of colour c now in the sky.
var phase: [Colour]Int = 0
var width: [Colour]Int = 0
let events = [0..n] _ =>
    let kind: 1..=3 = read()
    let t: 0..=2_000_000_000 = read()
    match kind:
        1 =>
            let (c, l, r, d): (Colour, 0..=len, 0..=len, -1..=1) = read()
            assert l <= r and d != 0
            phase[c] = ((if d == 1: l else: p - l) - t) % p
            width[c] = r - l
            Event.Cloud(phase[c], width[c], 1)
        2 =>
            let (l, r): (0..=len, 0..=len) = read()
            Event.Ask(t, l, r)
        _ =>
            let c: Colour = read()
            Event.Cloud(phase[c], width[c], -1)

var bit = Fenwick2D<PX, PY, Int>()

// The clouds in x in [xl, xr], y in [yl, yr], and in its copy shifted by -p (phase
// wraparound), on the plane with origin (ox, oy).
fn count(ox: Int, oy: Int, xl: Int, xr: Int, yl: Int, yr: Int) -> Int:
    var total = 0
    for s in [ox, ox + p]:
        let (a, b) = (max(xl - s, 0), min(xr - s, p - 1))
        let (c, d) = (max(yl - s + ox - oy, 0), min(yr - s + ox - oy, p + len - 1))
        if a <= b and c <= d:
            total += bit.fold(PX(a)..=PX(b), PY(c)..=PY(d))
    total

var caught: List<Int> = []
for plane in 0..2:
    let (ox, oy) = if plane == 0: (0, 0) else: (p, p - len)
    bit.clear()
    var asked = 0
    for e in events:
        match e:
            Cloud(x, w, v) =>
                let y = if plane == 0: x + w else: x - w + p
                bit.add(PX(x), PY(y - oy), v)
            Ask(t, l, r) =>
                let base = (-t) % p
                let got = if plane == 0: count(ox, oy, base, base + r, base + l, 3 * p - 1)
                    else: count(ox, oy, base + p + max(r + 1, p - r), base + 2 * p - 1, 0, base + 2 * p - l)
                if plane == 0: caught.push(got) else: caught[asked] += got
                asked += 1

for k in caught: print(k)
