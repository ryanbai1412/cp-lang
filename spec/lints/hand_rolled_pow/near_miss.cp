let x0: 1..=1000000000 = read()
fn halvings(start: Int) -> Int:
    var x = start
    var c = 0
    while x > 0:
        x /= 2
        c += 1
    return c
print(halvings(x0))
