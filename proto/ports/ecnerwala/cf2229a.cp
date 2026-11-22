// Codeforces 2229 A — Half the Span
//
// Each operation moves every element by at most 1, reducing `max(a) - min(a)`
// by at most 2 per step. Hence at least `ceil((max(a) - min(a)) / 2)` steps
// are required, and moving every element toward the midpoint achieves it.

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]-10 ** 9..=10 ** 9 = read()
print(ceil_div(a.max()! - a.min()!, 2))
