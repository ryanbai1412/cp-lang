// expect: ok, output "YES"
// model: MemoArray
// Against brute force: `@memo(array)` keeps its results in a `MemoArray` over its keys' range
// types, ranges not from 0 among them, and answers every key as the plain recursion does.
@memo(array)
fn paths(r: 0..=12, c: 3..=9) -> Int = if r == 0 or c == 3: 1 else: paths(r - 1, c) + paths(r, c - 1)
fn slow(r: Int, c: Int) -> Int = if r == 0 or c == 3: 1 else: slow(r - 1, c) + slow(r, c - 1)
var ok = true
for r in 0..=12:
    for c in 3..=9:
        ok = ok and paths(r, c) == slow(r, c)
print(yes(ok))
