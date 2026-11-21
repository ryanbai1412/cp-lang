use std.interval.{IntervalMap}

// Matched IntervalMap bench: 200k random range assigns with coalescing + on_old accumulation.
fn run():
    let n = 1000000
    domain X = 0..n
    var im = IntervalMap<X, Int>.new(0)
    var s: U64 = U64(987654321)
    var total = 0
    for q in 0..200000:
        s = s * U64(6364136223846793005) + U64(1442695040888963407)
        let a = Int((s >> 33) % U64(n))
        s = s * U64(6364136223846793005) + U64(1442695040888963407)
        let len = Int((s >> 33) % U64(5000)) + 1
        let r = min(n, a + len)
        let val = q % 17 + 1
        im.assign(X(a)..=X(r - 1), val, on_old: (seg, old) => do:
            total += seg.len * old
        )
    print(total)

run()
