type Z = Mod<1e9+7>

@!multitest
let a: [read(1..=2e5)]1..=1e9 = read()

// after each step two of P, Q, R are equal (say x) and the third is the prefix xor
var ways: Map<Int, Z> = [0: 1] default 0
var pre = 0
for x in a:
    ways[pre] = 3 * ways[pre] + 2 * ways[pre ^ x]
    pre ^= x
print(sum(ways.values))
