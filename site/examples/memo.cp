let n: 1..=20 = read()
domain I = 0..n

// @memo caching is prototype-only. This runnable version uses plain recursion.
fn f(i: I) -> Int:
    if let p = i.prev:
        return f(p) * 3 + i.to_idx
    1

print(f(I.last))
