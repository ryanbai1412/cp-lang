// cp: 0.1 a02484b+dirty
let n: 1..=1e6 = read()
domain I = 0..n

@memo(order: ascending)
fn f(i: I) -> Int:
    if let p = i.prev: return (f(p) * 3 + i.idx) % 1000000007
    1

print(f(I.last!))
