// expect: ok, output "3 24 7\n20 6"
// rule: R06.4.1
// A parameter's range may end at an earlier parameter, give or take a literal: its type is the
// range that reaches over the earlier one's, and the body sees it at its written range. A
// `@memo(array)` table keys on those reaches.
fn f(n: 0..=30, k: 0..=n) -> Int = n - k
fn g(i: 1..=5, j: i..=i + 2) -> Int:
    i * 10 + j
fn outer() -> Int:
    fn h(a: 0..=9, b: 0..a) -> Int = a + b
    h(4, 3)
print(f(5, 2), g(2, 4), outer())
@memo(array)
fn c(n: 0..=10, k: 0..=n) -> Int = if k == 0 or k == n: 1 else: c(n - 1, k - 1) + c(n - 1, k)
@memo(array)
fn paths(r: 0..=4, s: r..=4) -> Int = if r == s: 1 else: paths(r + 1, s) + 1
print(c(6, 3), paths(0, 4) + paths(4, 4))
