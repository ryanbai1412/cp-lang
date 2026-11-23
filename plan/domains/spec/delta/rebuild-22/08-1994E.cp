// cf1994E Wooden Game: a tree of size n can be cut into removals of any sizes summing to n
// (peel leaves), so only the sizes matter. Greedy from the top bit: a size bit the answer
// already has lets the rest of that tree fill every lower bit.
@!multitest
let k: 1..=1e6 = read()
var best = 0
repeat k:
    let n: 1..=1e6 = read()
    let _parents = read<[n - 1]1..=n>()
    for b in (0..20).rev if (n >> b) & 1 == 1:
        if (best >> b) & 1 == 1:
            best |= (1 << b) - 1
            break
        best |= 1 << b
print(best)
