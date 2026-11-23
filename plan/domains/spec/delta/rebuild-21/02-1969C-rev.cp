// CF 1969C, revised: copies are directional, at most kl copies from a left neighbour and at
// most kr from a right neighbour (input "n kl kr", kl, kr <= 5). A block of length len keeps
// a[p]; cells before p copy from the right, cells after p from the left.
@!multitest
let n = read<1..=3e5>()
let kl = read<0..=5>()
let kr = read<0..=5>()
let a = read<[n]1..=1e9>()

// dp[i, jl, jr]: least sum of the first i elements using at most jl left- and jr right-copies
var dp: [0..=n, 0..=kl, 0..=kr]?Int = None
for jl in 0..=kl:
    for jr in 0..=kr: dp[0, jl, jr] = 0
for i in 1..=n:
    for len in 1..=min(i, kl + kr + 1):
        for p in i - len..i:
            let right = p - (i - len)
            let left = i - 1 - p
            for jl in left..=kl:
                for jr in right..=kr:
                    dp[i, jl, jr] min= dp[i - len, jl - left, jr - right] + len * a[p]
print(dp[n, kl, kr]!)
