// CCC 2018 S4 Balanced Trees, as it has to be written today: the blocks of
// equal quotients are walked by hand. Builds; stress OK against 4329704.cpp.
@memo
fn trees(w: Int) -> Int:
    if w == 1: return 1
    var total = 0
    var k = 2
    while k <= w:
        let q = w / k
        let last = w / q
        total += (last - k + 1) * trees(q)
        k = last + 1
    total
let n: 1..=1e9 = read()
print(trees(n))
