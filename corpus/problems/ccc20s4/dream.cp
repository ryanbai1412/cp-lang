// ccc20s4 Swapping Seats: fewest swaps to seat A, B and C people in three blocks around a round
// table. Try every rotation and both orders of B and C; for each, count how many of letter x sit
// in letter y's block. A two-cycle costs one swap and a three-cycle two.
let s = read<Str>()
let n = s.len
let size = [3] k => s.count(c => c == "ABC"[k])
let pre = [3] k => (s + s).map(c => Int(c == "ABC"[k])).scan(+)
fn swaps(start: Int, order: [3]Int) -> Int:
    var m: [3, 3]Int = 0
    var at = start
    for y in order:
        for x in 0..3:
            m[x, y] = pre[x][at + size[y]] - pre[x][at]
        at += size[y]
    var total = 0
    for (x, y) in [(0, 1), (0, 2), (1, 2)]:
        let two = min(m[x, y], m[y, x])
        total += two
        m[x, y] -= two
        m[y, x] -= two
    total + 2 * (m[0, 1] + m[0, 2])
print((0..n).map(i => min(swaps(i, [0, 1, 2]), swaps(i, [0, 2, 1]))).min()!)
