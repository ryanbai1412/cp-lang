// expect: ok, output "YES\nYES"
var seed = 12345
fn rnd(m: Int) -> Int:
    seed = (seed * 1103515245 + 12345) % 2147483648
    seed / 65536 % m
var ok = true
for _ in 0..300:
    let n = 1 + rnd(9)
    let a = [rnd(4) for _ in 0..n]
    var got: List<(Int, Int)> = []
    for r in a.valleys(): got.push((r.first!, r.last!))
    var want: List<(Int, Int)> = []
    for l in 1..n:
        for r in l..n - 1:
            var good = true
            for k in l..=r:
                if not (a[k] < a[l - 1] and a[k] < a[r + 1]): good = false
            if good: want.push((l, r))
    if got != want: ok = false
print(yes(ok))
let samp = [3, 1, 2, 5]
print(yes(samp.valleys().collect() == [1..2, 1..3]))
