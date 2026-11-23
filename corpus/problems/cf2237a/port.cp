// Source: Codeforces 2237A (Destroying Towers), https://codeforces.com/contest/2237/problem/A
// Problem: n <= 100 towers of heights a[i] <= 1000. Operate on each tower once, in any order:
//   from tower i a laser cuts the first taller tower to its right down to tower i's height.
//   Print the least possible final sum of heights. Multitest.
// Solution: a tower is only cut to the height of a tower on its left, so it never ends below
//   its prefix minimum; operating left to right gets every tower there (tower i, already at
//   the prefix minimum, cuts tower i + 1 if it is taller). Print the sum of prefix minima:
//   `a.scan(min)` holds them, O(n).

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]1..=10 ** 9 = read()
let low = a.scan(min)
print(sum(low.prefix(i) for i in 1..=n))
