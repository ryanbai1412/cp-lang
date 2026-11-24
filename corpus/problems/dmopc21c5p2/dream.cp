// DMOPC 2021 C5 P2: construct the permutation; 1 3 2 4 5 ... n works for every n except 2.
let n = read<1..=1_000_000>()
if n == 2:
    print(-1)
else:
    print(chain([1, 3, 2], 4..=n).take(n))
