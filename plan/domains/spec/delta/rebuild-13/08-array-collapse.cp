// Codeforces 1913D Array Collapse
// in: t; per test n, then n distinct ints p; out: per test, the number of reachable arrays modulo 998244353
type Z = Mod<998244353>

@!multitest
let n: 1..=3e5 = read()
let p = read<[n]1..=1e9>()

// ways[i]: arrays whose last kept element is p[i]. The previous kept element is
// either between the nearest smaller one l and i, or a prefix minimum before l.
var ways: [n]Z = 0
var before: [n + 1]Z = 0
var stack = List<Int>()
var on_stack: Z = 0
for i in p.keys:
    while let j = stack.last and p[j] > p[i]:
        on_stack -= ways[j]
        stack.pop()
    ways[i] = match stack.last:
        None => before[i] + 1
        Some(l) => before[i] - before[l] + on_stack - ways[l]
    on_stack += ways[i]
    stack.push(i)
    before[i + 1] = before[i] + ways[i]
print(on_stack)
