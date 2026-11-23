let n: 1..=100 = read()
var k = 3  // expect: var_never_reassigned
var total = 0
for i in 0..n:
    total += i % k
print(total)
