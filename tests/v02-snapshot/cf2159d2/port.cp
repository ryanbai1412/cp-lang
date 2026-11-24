// Source: Codeforces 2159D2 (Inverse Minimum Partition, Hard Version),
//   https://codeforces.com/contest/2159/problem/D2
// Problem: a segment costs ceil(last / min); f(c) is the cheapest total cost of cutting c
//   into segments. Print the sum of f over all subarrays of a (n <= 4e5, a_i <= 1e18).
//   Multitest, sum of n <= 4e5.
// Solution: a last segment of cost above 3 never beats splitting it, and f(a[l..=i]) only
//   grows as l moves left, so starts[z], the number of left ends with f > z, describes every
//   f. The suffix minima of a[..=i] form an increasing stack, each entry with its starts. For
//   x = a[i]: a cost-1 last segment starts after the previous smaller value (its starts,
//   shifted by one); cost c in {2, 3} starts after the last value < ceil(x / c), found by
//   binary search, giving starts[z] min= that entry's starts[z - c]. Add sum(starts). The
//   lists hold O(log max a) entries: O(n log max a).
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
