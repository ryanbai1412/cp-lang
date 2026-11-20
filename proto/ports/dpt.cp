// cp: 0.1 1f59ac0+dirty
// AtCoder DP T Permutation: count permutations of 1..=n with the given
// pattern of '<' and '>'. ways[j]: the prefix ends at its (j+1)-th smallest
// value; each sign turns into a prefix or suffix sum.
const P = 1000000007
type Z = Mod<P>
let n: 2..=3000 = read()
domain Sign = 1..n
fn random_signs() -> [Sign]Char: [Sign] _ => if rand(0, 1) == 0: '<' else: '>'
let signs: [Sign]Char = read(gen: random_signs)

domain Rank = 0..n
var ways: [Rank]Z = Z(0)
ways[Rank.first] = Z(1)
for s in Sign:
    let live = Rank.first..=Rank(s.label)   // ranks the longer prefix can end at
    var next: [Rank]Z = Z(0)
    var acc = Z(0)
    if signs[s] == '<':
        for r in live:
            next[r] = acc
            acc += ways[r]
    else:
        for r in live.rev:
            acc += ways[r]
            next[r] = acc
    ways = next
print(sum(ways[r] for r in Rank))
