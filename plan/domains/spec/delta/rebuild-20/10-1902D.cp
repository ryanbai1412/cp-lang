// CF 1902D Robot Queries. With prefix positions P_0..P_n, reversing l..r
// leaves P_0..P_{l-1} and P_r..P_n alone and maps the middle points to
// P_{l-1} + P_r - P_k for k in l-1..=r. So (x, y) is visited iff it is some
// P_k with k outside l..r-1, or P_{l-1} + P_r - (x, y) is some P_k with
// l-1 <= k <= r. Each point keeps its sorted list of indices.
let n: 1..=2e5 = read()
let q: 1..=2e5 = read()
let s: Str of "UDLR" = read()
var pos: List<(Int, Int)> = [(0, 0)]
for ch in s:
    let (x, y) = pos.last!
    pos.push(match ch:
        'U' => (x, y + 1)
        'D' => (x, y - 1)
        'L' => (x - 1, y)
        _ => (x + 1, y))
let at = (0..=n).group(by: k => pos[k])
fn seen(pt: (Int, Int), lo: Int, hi: Int) -> Bool:
    if let ks = at.get(pt):
        let c = Int(ks.partition_point(k => k >= lo))
        return c < ks.len and ks[c] <= hi
    false
for _ in 0..q:
    let (x, y, l, r) = read<(-n..=n, -n..=n, 1..=n, 1..=n)>()
    let (ax, ay) = pos[l - 1]
    let (bx, by) = pos[r]
    let hit = (seen((x, y), 0, l - 1) or seen((x, y), r, n)
        or seen((ax + bx - x, ay + by - y), l - 1, r))
    print(yes(hit))
