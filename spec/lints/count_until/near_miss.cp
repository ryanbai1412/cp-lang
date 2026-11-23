fn halvings(n: Int) -> Int:
    var x = n
    var k = 0
    while x > 1:
        x = (x + 1) / 2
        k += 1
    k

let n: 1..=1000000000 = read()
print(halvings(n))
