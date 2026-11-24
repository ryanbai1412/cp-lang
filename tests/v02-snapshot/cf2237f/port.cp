// Source: Codeforces 2237F, https://codeforces.com/contest/2237/problem/F
// Problem: multitest; a of n values in 1..=m (m <= n <= 5e5, sum n <= 5e5). A paint writes
//   1, 2, .., m over a length-m interval, later paints overwrite earlier ones; a is valid if
//   some paints cover every position. Print the fewest elements to change to make a valid.
// Solution: maximise the kept elements. A kept a[i] belongs to the paint starting at
//   st = i - (a[i] - 1) in 0..=n - m, and some paint must survive whole (possibly rewritten).
//   One left-to-right DP: ending_at[i] (best with i kept), at_boundary (its prefix max, to
//   open a paint at st), after_block (a whole paint already ended) and last_with_start[st]
//   (the previous kept index of the same paint). Print n - best; O(n).

@!multitest
let n: 1..=500_000 = read()
let m: 1..=n = read()
let a: [0..n]1..=m = read()
var at_boundary: [0..=n]Int = 0
var after_block: [0..=n]?Int = None
var ending_at: [0..=n]?Int = None
var last_with_start: [0..=n - m]?Int = None

after_block[m] = 0
for i in 0..n:
    at_boundary[i + 1] max= at_boundary[i]
    after_block[i + 1] max= after_block[i]
    let block_start = i - (a[i] - 1)
    if 0 <= block_start <= n - m:
        ending_at[i] max= at_boundary[block_start] + 1
        if let v = after_block[i]: ending_at[i] max= v + 1
        if let p = last_with_start[block_start] and let v = ending_at[p]:
            ending_at[i] max= v + 1
        last_with_start[block_start] = i
        after_block[block_start + m] max= ending_at[i]
        if let v = ending_at[i]: at_boundary[i + 1] max= v
print(n - max(at_boundary[n - m], after_block[n] ?? 0))
