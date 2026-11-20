// cp: 0.1 5df8551
let n: 1..=5000 = read()
domain I = 0..n
@memo
fn f(i: I, j: I) -> Int:
    if let a = i.prev:
        if let b = j.prev:
            return (f(a, j) + f(i, b)) % 1000000007
    1
print(f(I.last, I.last))
