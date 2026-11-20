// Educational DP T, Permutation: count the permutations of 1..=n in which
// neighbours i and i+1 compare the way the i-th sign says.
// original: corpus/submissions/4131863.cpp
const P = 1000000007
type Z = Mod<P>
let n: 2..=3000 = read()
domain Rank = 0..n
domain Gap = 1..n
fn random_signs() -> [Gap]Char: [Gap] _ => if rand(0, 1) == 0: '<' else: '>'
let signs: [Gap]Char = read(gen: random_signs)

// ways[r]: orderings of the values placed so far in which the last one has
// r of the others below it. A new value with r below it must follow a value
// that had fewer than r below it ('<'), or at least r ('>').
var ways: [Rank]Z = 0
ways[Rank.first] = 1
for g in Gap:
    let placed = Rank(g.label)          // how many values are placed already
    var next: [Rank]Z = 0
    var run: Z = 0
    if signs[g] == '<':
        for r in ..=placed:
            next[r] = run
            run += ways[r]
    else:
        for r in (..placed).rev:
            run += ways[r]
            next[r] = run
    ways = next
print(sum(ways))
