let n: 1..=20 = read()
domain I = 0..n

@memo
fn f(i: I) -> Int:
    if let p = i.prev:
        return f(p) * 3 + i.idx
    1

print(f(I.last))
