// Count the subarrays of `a` whose sum is at most `k`.
let (n, k): (1..=2e5, 0..=1e14) = read()
domain I = 1..=n
let a: [I]Int = read()

var pre: [I.cuts]Int = 0
for i in I: pre[i.after] = pre[i.before] + a[i]

@brute(count_brute)
fn count() -> Int:
    var total = 0
    var lo = I.first
    for hi in I:
        while pre[hi.after] - pre[lo.before] > k: lo = lo.next!
        total += hi - lo + 1
    total

fn count_brute() -> Int:
    sum(1 for l in I for r in I if l <= r and pre[r.after] - pre[l.before] <= k)

let answer = count()
print("{answer}")
