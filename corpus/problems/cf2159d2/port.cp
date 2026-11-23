// Codeforces 2159 D2: Inverse Minimum Partition (Hard), summed over all
// subarrays.
//
// As in D1, a partition's last segment ends at i and costs l when every value
// in it is at least ceil(a[i] / l), and costs of 2 and 3 are the only ones
// worth trying (a longer segment splits no worse). The suffix minima of the
// prefix form a strictly increasing stack, and each entry keeps `starts`:
// `starts[z]` is how many left ends l make the cheapest partition of
// a[l..=i] cost more than z. Summing `starts` sums those costs over every
// left end, and a list ends at its first 0.
@!multitest
let n: Int = read()
let a: [0..n]Int = read()

var stack: List<(val: Int, starts: List<Int>)> = []
var ans = 0
for (i, x) in a.items:
    while let top = stack.last and top.val >= x: stack.pop()
    var starts = [i + 1] + (stack.last.map(t => t.starts) ?? [0])
    for l in 2..=3 if l < starts.len:
        let below = (0..stack.len).first_true(k => stack[k].val >= ceil_div(x, l)) ?? stack.len
        let prev = if below > 0: stack[below - 1].starts else: [0]
        starts = starts[..l] + [min(u, v) for (u, v) in zip(starts[l..], prev)]
    ans += starts.sum()
    stack.push((x, starts))
print(ans)
