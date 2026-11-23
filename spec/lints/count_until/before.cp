fn ceil_log3(n: Int) -> Int:
    var p = 1
    var k = 0
    while p < n:
        p *= 3
        k += 1
    k

let n: 1..=1000000000 = read()
print(ceil_log3(n))
