@!interactive
@!multitest
let n: Int = read()
var cost = [0 for _ in 0..n]
var good = [1 for _ in 0..n]
for cur in (0..n).rev:
    let (a, k): (Int, Int) = read()
    for _ in 0..k:
        let nxt = read::<Int>() - 1
        cost[cur] += cost[nxt]
        good[cur] = lcm(good[cur], gcd(a, good[nxt]))
    if good[cur] == 1:
        cost[cur] += 1
        good[cur] = a
    print(cost[cur])
