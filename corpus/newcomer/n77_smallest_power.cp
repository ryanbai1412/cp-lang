// Smallest k with 3^k >= n, written as a C++ loop.
let n: 1..=1_000_000_000 = read()
fn ceil_log3(n: Int) -> Int:
    var p = 1
    var k = 0
    while p < n: // expect: missing smallest_k_loop
        p *= 3
        k += 1
    k
print(ceil_log3(n))
