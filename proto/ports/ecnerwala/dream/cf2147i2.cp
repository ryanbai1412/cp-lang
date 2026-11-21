// Codeforces 2147 I2: Longest Increasing Path, hard (dream).
// Clusters of k consecutive points sit at bases U, 2U, 4U, ... (U = 3k), so
// the gaps base(j) - base(i), ordered by j then by falling i, grow by at least U.
// A rally between clusters j > i bounces hub t -> partner k-2-t -> hub t+1:
// 2k - 1 jumps of gap + (2-k ..= k-1), one longer each time. Rallies run in gap
// order; one bridge point links consecutive rallies of hub j, and hub j's last
// rally ends one jump short of hub j+1. C(c,2)(2k-1) + C(c-1,2) points on
// ck + C(c-1,2) values. A short path needs only one hub: out d+1, on d+2, home.
let n, m: Int = read()

fn one_hub(n: Int) -> List<Int>:          // 0 1 3 0 4 9 0 10 ...
    var path = [0]
    var jump = 0
    while path.len < n:
        let out = jump + 1
        jump = out + jump + 2
        path.extend([out, jump, 0])
    path.take(n)

fn rallies(n: Int, m: Int) -> List<Int>:
    let k_for = c => (m - choose2(c - 1)) / c
    let c = (2..).first(c => choose2(c) * (2 * k_for(c) - 1) + choose2(c - 1) >= n)!
    let k = k_for(c)
    let base = i => 3 * k * 2 ** i
    var path: List<Int> = []
    for j in 1..c:
        for i in (0..j).rev:
            let gap = base(j) - base(i)
            for t in 0..k:
                path.push(base(j) + t)
                if t < k - 1: path.push(base(i) + k - 2 - t)
            if i > 0: path.push(base(j) + gap + 2 * k - 1)   // out gap + k, back gap + 2k - 1
    path.take(n)

print(if n - (n - 1) / 3 <= m: one_hub(n) else: rallies(n, m))
