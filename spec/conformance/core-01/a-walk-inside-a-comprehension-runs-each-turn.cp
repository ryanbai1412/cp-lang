// expect: ok, output "3 1 1 0\n3"
// rule: 01.8.2
// input: "5\n1 3 1 2 1\n"
// The count's walk of `cards` sits inside the comprehension's `for step` loop, so it runs four times: the C++ must not free `cards` after the first (noip10p2).
let m: 1..=120 = read()
let cards: [m]1..=4 = read()
let have = [count(c == step for c in cards) for step in 1..=4]
print(have)
domain A = 0..=have[0]
var best: [A]Int = 0
for a in A:
    best[a] = a.ord
print(best[A.last!])
