// CCC 2021 S3 Lunch Concert: friend i walks w per metre until within d of
// the concert at c. The total walking time is convex in c, so find where it
// stops falling.
use std.seq.{bsearch_first}

let n: 1..=2e5 = read()
let friends: [0..n](p: 0..=1e9, w: 1..=1000, d: 0..=1e9) = read()

fn walk(c: Int) -> Int = sum(f.w * max(0, abs(f.p - c) - f.d) for f in friends)

print(walk(bsearch_first(0..=1e9, c => walk(c + 1) >= walk(c))!))
