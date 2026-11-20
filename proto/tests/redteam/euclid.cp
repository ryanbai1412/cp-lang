// a == (a / b) * b + a % b and 0 <= a % b < |b|, for every sign and at the ends of Int
let t: 1..=50 = read()
for _ in 0..t:
    let a: Int = read()
    let b: Int = read()
    if b == 0:
        print(-1)
        continue
    print(a / b, a % b)
