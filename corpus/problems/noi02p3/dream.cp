// noi02p3 Nine-Headed Dragon: the big head eats exactly k fruits including the root, every other
// head at least one; a branch costs when one head eats both ends. With three or more heads the
// small heads alternate freely, so only "big or small" matters per fruit: a tree knapsack over
// how many fruits the big head eats, merged child by child as a min-plus convolution.
let (n, heads, k) = read<(1..=300, 2..=300, 1..=300)>()
domain F = 1..=n
let branches = read<[n - 1](a: F, b: F, cost: 0..=100_000)>()
let t = Rooted(Graph.undirected(branches), root: F.first!)
var up: [F]Int = 0
for br in branches:
    up[if t.parent(br.a) == br.b: br.a else: br.b] = br.cost
let tropical = Semiring(min, +)
fn clash(w: 0..2, wc: 0..2, c: F) -> Int:
    if w == wc and (w == 1 or heads == 2): up[c] else: 0
// eat(v)[w][x]: the cheapest for v's subtree when head w (0 small, 1 big) eats v and the big head
// eats x of its fruits.
fn eat(v: F) -> [0..2]List<?Int>:
    var best: [0..2]List<?Int> = [[some(0), none], [none, some(0)]]
    for c in t.children(v):
        let sub = eat(c)
        let via = (w, wc) => convolve(best[w], sub[wc].map(x => x + clash(w, wc, c)).collect(), over: tropical, shape: Curved.Neither)
        best = [0..2] w => zip(via(w, 0), via(w, 1)).map(((p, q)) => min(p, q)).collect()
    best
print(if heads - 1 + k > n: -1 else: eat(t.root)[1][k]!)
