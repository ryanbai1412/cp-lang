type Z = Mod<1_000_000_007>

@!multitest
let n: Int = read()
let a: [0..n]Int = read()

// after each step two of P, Q, R are equal (say x) and the third is the prefix xor;
// ways[x] counts those states, and only ways[previous prefix] changes per step
var ways: HashMap<Int, Z> = [0: 1]
var pre = 0
for x in a:
    ways[pre] = 3 * (ways.get(pre) ?? 0) + 2 * (ways.get(pre ^ x) ?? 0)
    pre ^= x
print(sum(v for (_, v) in ways))
