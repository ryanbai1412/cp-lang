use std.window.{MonoDeque}

// Matched MonoDeque bench: 2M sliding-window minimums (window size 1000).
let n = 2000000
let w = 1000
var mq = MonoDeque<Int>.new(by: min)
var s: U64 = U64(246813579)
var ans = 0
for i in 0..n:
    s = s * U64(6364136223846793005) + U64(1442695040888963407)
    let v = Int((s >> 33) % U64(1000000))
    mq.push(i, v)
    mq.drop_before(i - w + 1)
    if let b = mq.best:
        ans += b
print(ans)
