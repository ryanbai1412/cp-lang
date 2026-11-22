@!multitest
let a: [0..read(1..=2e5)]1..=1e9 = read()

// after each step two of P, Q, R are equal (say x) and the third is the prefix xor;
// ways[x] counts those states, and only ways[previous prefix] changes per step
var ways: HashMap<Int, Mod<1e9 + 7>> = [0: 1]
var pre = 0
for x in a:
    ways[pre] = 3 * (ways.get(pre) ?? 0) + 2 * (ways.get(pre ^ x) ?? 0)
    pre ^= x
print(ways.values.sum())
