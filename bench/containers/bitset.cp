// Matched Bitset bench: 200k-bit subset-sum knapsack + 1000 shift_or + &-chain popcounts.
domain B = 0..200001
var weights: List<Int> = []
var s: U64 = U64(123456789)
for i in 0..4000:
    s = s * U64(6364136223846793005) + U64(1442695040888963407)
    weights.push(Int((s >> 33) % U64(100)) + 1)

let reach = Bitset<B>.subset_sums(weights)
var a = reach.shifted(0)
var b = reach.shifted(37)
var c = reach.shifted(-19)
var ans = reach.len
for step in 0..200:
    a.shift_or(step % 31 + 1)
    ans += (a & b & c).len
print(ans)
