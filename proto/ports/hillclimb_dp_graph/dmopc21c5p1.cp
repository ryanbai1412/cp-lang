// cp: 0.1 0f8d77b+dirty
// DMOPC '21 Contest 5 P1 Permutations & Primes: arrange 1..=n so that no two
// neighbours differ by a prime... the odd numbers, then 5 and 4, then the evens.
let n: 1..=1e6 = read()
if n == 1: print(1)
elif n <= 4: print(-1)
else:
    let odds = [x for x in 1..=n if x % 2 == 1 and x != 5]
    let evens = [x for x in 1..=n if x % 2 == 0 and x != 4]
    print(odds + [5, 4] + evens)
