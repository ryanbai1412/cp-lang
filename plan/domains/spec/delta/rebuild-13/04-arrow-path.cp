// Codeforces 1948C Arrow Path
// in: t; per test n, then two rows of n characters '<' or '>'; out: per test YES if (2, n) is reachable from (1, 1), else NO
@!multitest
let n: 2..=2e5 = read()
domain R = 0..2
domain C = 0..n
let g = read<[R, C]Char>()

var seen: [R, C]Bool = false
seen[R(0), C(0)] = true
var queue = Deque.of([(R(0), C(0))])
while let p = queue.pop_front():
    for (r, c) in p.adj4:
        let (tr, tc) = (r, c).step(if g[r, c] == '<': Dir4.left else: Dir4.right)!
        if not seen[tr, tc]:
            seen[tr, tc] = true
            queue.push((tr, tc))
print(yes(seen[R(1), C(n - 1)]))
