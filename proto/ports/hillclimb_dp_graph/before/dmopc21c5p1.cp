// DMOPC '21 Contest 5 P1 Permutations & Primes: arrange 1..=n so that no two
// neighbours differ by a prime... the odd numbers, then 5 and 4, then the evens.
let n: 1..=1e6 = read()
if n == 1: print(1)
elif n <= 4: print(-1)
else:
    var order: List<Int> = []
    for x in 1..=n:
        if x % 2 == 1 and x != 5: order.push(x)
    order.push(5)
    order.push(4)
    for x in 1..=n:
        if x % 2 == 0 and x != 4: order.push(x)
    print(order)
