// AtCoder DP T Permutation: count the permutations of 1..=n whose neighbours
// compare as the given string of '<' and '>' says. ways[j]: the prefix so far
// ends at its (j + 1)-th smallest value, so '<' sums the ranks below and '>'
// the ranks from here up.
type Z = Mod<1_000_000_007>
let n: 2..=3000 = read()
let signs: Str = read()

var ways: [0..n]Z = 0
ways[0] = 1
for len in 0..n - 1:                    // the prefix has len + 1 values
    var next: [0..n]Z = 0
    var acc: Z = 0
    if signs[len] == '<':
        for j in 1..=len + 1:
            acc += ways[j - 1]
            next[j] = acc
    else:
        for j in (0..=len).rev:
            acc += ways[j]
            next[j] = acc
    ways = next
var total: Z = 0
for w in ways: total += w
print(total)
