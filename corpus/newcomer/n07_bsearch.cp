let (n, k): (1..=200000, 1..=1000000000) = read()
let a: [0..n]1..=1000000000 = read()
fn ok(x: Int) -> Bool:
    var c = 0
    for v in a:  // expect: loop_reduction
        c += v / x
    return c >= k
var lo = 1
var hi = 1000000001
while lo < hi:  // expect: hand_rolled_answer_search
    let mid = (lo + hi) / 2
    if ok(mid):
        lo = mid + 1
    else:
        hi = mid
print(lo - 1)
