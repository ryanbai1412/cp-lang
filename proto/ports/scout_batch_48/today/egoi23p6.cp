// EGOI '23 P6 (Candy): fewest adjacent swaps that bring the first m boxes to
// a total of at least k. Runnable on proto/cpc today.
// original: corpus/submissions/5678056.cpp
let n: 1..=100 = read()
let m: 1..=n = read()
let k: 1..=1e11 = read()
domain Box = 0..n
let a: [Box]0..=1e9 = read()

let cap = n * n / 2 + 10
domain Take = 0..=m
domain Pos = 0..=n
domain Swaps = 0..cap
// best[i, j, s]: most candy from boxes j.. placed into slots i..m-1 with s swaps
var best: [Take, Pos, Swaps]Int = 0
for i in (0..m).rev:
    for j in (i..n).rev:
        for s in (j - i)..cap:
            best[Take(i), Pos(j), Swaps(s)] = max(best[Take(i), Pos(j + 1), Swaps(s)],
                a[Box(j)] + best[Take(i + 1), Pos(j + 1), Swaps(s - (j - i))])

var ans: ?Int = none
for s in 0..cap:
    if ans == none and best[Take(0), Pos(0), Swaps(s)] >= k: ans = s
if let s = ans: print(s)
else: print("NO")
