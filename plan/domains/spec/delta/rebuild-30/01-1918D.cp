// cf1918d Blocking Elements: binary search the cost X. Block 0 and n + 1 (value 0);
// dp[i] = a[i] + min dp[j] over blocked j < i whose gap j+1..i-1 sums to at most X.
// The gap's left end comes from two pointers over a `Window(+)`, the min
// over the allowed j from a sliding `Window(min)` (monotone deque).
@!multitest
let n: 1..=1e5 = read()
let raw = read<[n]1..=1e9>()
domain P = 0..=n + 1
let a: [P]Int = [P] i => if i == P.first! or i == P.last!: 0 else: raw[Int(i) - 1]

fn feasible(x: Int) -> Bool:
    var dp: [P]Int = 0
    var gap = Window(+)
    var lo = P.first!
    var w = Window(min)
    var front = P.first!
    for i in P.after(P.first!):
        let j = i.prev!
        w.push(dp[j])
        gap.push(a[j])
        while (gap.all ?? 0) > x:
            gap.pop()
            lo = lo.next!
        while front.next! < lo:
            w.pop()
            front = front.next!
        dp[i] = w.all! + a[i]
    dp[P.last!] <= x

print((0..=a.sum()).first_true(feasible)!)
