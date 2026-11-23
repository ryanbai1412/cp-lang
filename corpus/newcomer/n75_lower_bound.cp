let (n, q): (1..=200000, 1..=200000) = read()
let a: [0..n]Int = read()
let b = a.sorted()
for _ in 0..q:
    let x: Int = read()
    var lo = 0
    var hi = n
    while lo < hi:  // expect: hand_rolled_answer_search
        let mid = (lo + hi) / 2
        if b[mid] < x:
            lo = mid + 1
        else:
            hi = mid
    print(n - lo)
