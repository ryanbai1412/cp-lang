// Matched Deque bench vs std::deque:
// 200k prefill + 4M balanced push/push_front/pop_front/pop + 2M random index reads.
var dq: Deque<Int> = []
dq.reserve(200000)
for i in 0..100000:
    dq.push(i)
    dq.push_front(i)
var rng: U64 = U64(271828182)
var acc = 0
for i in 0..4000000:
    rng = rng * U64(6364136223846793005) + U64(1442695040888963407)
    let r = Int((rng >> 33) & U64(7))
    if r < 2:
        dq.push(i)
    elif r < 4:
        dq.push_front(i)
    elif r < 6:
        if let v = dq.pop_front(): acc += v
    else:
        if let v = dq.pop(): acc += v
let n = dq.len
for _ in 0..2000000:
    rng = rng * U64(6364136223846793005) + U64(1442695040888963407)
    let idx = Int((rng >> 33) % U64(n))
    acc += dq.get(idx)
print(acc + dq.len)
