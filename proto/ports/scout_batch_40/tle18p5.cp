// TLE '18 P5, longest increasing subsequence behind a judge header
// original: corpus/submissions/5876770.cpp
// The judge's header supplies the length N, get_item and print_num; the
// submission has no includes and no stdin. Print the length, then one
// longest strictly increasing subsequence.
// WISH: `@grader const` for a constant the judge defines (see backlog C1).
@grader const N: 0..=2000
@grader fn get_item(i: Int) -> Int
@grader fn print_num(x: Int)

domain I = 0..N
let a: [I]Int = [I] i => get_item(i.to_idx)
var len: [I]Int = 1
var prev: [I]?I = none
for i in I:
    for j in ..i if a[j] < a[i] and len[j] >= len[i]:
        (len[i], prev[i]) = (len[j] + 1, j)

// back along `prev` from where a longest one ends (`at` is none when N = 0)
var path: List<Int> = []
var at = max_by(I, i => len[i])
while let i = at:
    path.push(a[i])
    at = prev[i]
reverse(mut path)
print_num(path.len)
for x in path: print_num(x)
