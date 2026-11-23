// CF 2001B Generate Permutation
// in: t; per test n. out: a permutation with equal carriage returns for both typewriters, or -1.
@!multitest
let n = read<1..=2e5>()
if n % 2 == 0:
    print(-1)
    return
print(chain((1..=n by 2).rev, 2..n by 2))
