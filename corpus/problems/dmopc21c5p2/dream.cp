// DMOPC 2021 C5 P2: a permutation of 1..=n where no adjacent pair differs by... swap 2 and 3.
let n = read Int
if n == 2:
    print(-1)
else:
    print(chain([1, 3, 2], 4..=n).take(n))
