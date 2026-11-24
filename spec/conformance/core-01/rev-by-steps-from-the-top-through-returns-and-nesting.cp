// expect: ok, output "9 9 -1 -1\n11 8 5 2\n312010"
// rule: 01.8.4
// `(a..b).rev by k` steps down from the top: an early return, a comprehension and a nested walk whose top moves with the outer value.
fn last_odd(n: Int, k: Int) -> Int:
    for i in (0..n).rev by k:
        if i % 2 == 1: return i
    -1
print(last_odd(10, 3), last_odd(10, 4), last_odd(1, 5), last_odd(0, 2))
let xs = [i for i in (2..=11).rev by 3]
print(xs)
var acc = 0
for i in (0..4).rev by 1:
    for j in (0..=i).rev by 2: acc = acc * 10 + j
print(acc)
