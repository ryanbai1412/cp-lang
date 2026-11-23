// Matched sliding-window bench: 2M sliding-window minimums (window size 1000).
// `Window(min)` keeps a monotone deque for a selective op, as the C++ does.
let n = 2000000
let w = 1000
var mq = Window(min)
var ring: [0..w]Int = 0  // the last `w` values, so `pop` can name the one leaving
var s: U64 = U64(246813579)
var ans = 0
for i in 0..n:
    s = s * U64(6364136223846793005) + U64(1442695040888963407)
    let v = Int((s >> 33) % U64(1000000))
    mq.push(v)
    if mq.len > w: mq.pop(ring[i % w])
    ring[i % w] = v
    if let b = mq.all:
        ans += b
print(ans)
