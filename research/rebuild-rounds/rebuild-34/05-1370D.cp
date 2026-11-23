// CF 1370D Odd-Even Subsequence: binary search the cost c. Cost <= c is
// reachable iff a greedy pick of length k exists with every odd (or every
// even) slot at most c.
let n: 2..=2e5 = read()
let k: 2..=n = read()
let a = read<[n]1..=1e9>()

fn fits(c: Int, odd: Bool) -> Bool:
    var len = 0
    for v in a:
        if (len % 2 == 0) == odd and v > c: continue
        len += 1
    len >= k

print((1..=1e9).first_true(c => fits(c, true) or fits(c, false))!)
