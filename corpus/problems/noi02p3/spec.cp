// noi02p3 Nine-Headed Dragon: the big head eats exactly k fruits including the root, every other
// head at least one; a branch costs when one head eats both ends. With three or more heads the
// small heads alternate freely, so only "big or small" matters per fruit: a tree knapsack over
// how many fruits the big head eats, merged child by child as a min-plus convolution.
let (n, heads, k) = read<(1..=300, 2..=300, 1..=300)>()
domain F = 1..=n
let branches = read<[n - 1](a: F, b: F, cost: 0..=100_000)>()
let t = Rooted(Graph.undirected(branches), root: F.first!)
let tropical = Semiring(min, +)
enum Head: Small, Big
fn clash(w: Head, wc: Head, c: F) -> Int:
    if w == wc and (w == Head.Big or heads == 2): t.up_arc(c)!.cost else: 0
// eat(v)[w][x]: the cheapest for v's subtree when head w eats v and the big head eats x of its
// fruits.
fn eat(v: F) -> [Head]List<?Int>:
    var best: [Head]List<?Int> = [Head] w => if w == Head.Big: [None, Some(0)] else: [Some(0), None]
    for c in t.children(v):
        let sub = eat(c)
        let via = (w, wc) => convolve(best[w], sub[wc].map(x => x + clash(w, wc, c)).collect(), over: tropical, shape: Curved.Neither)
        best = [Head] w => zip(via(w, Head.Small), via(w, Head.Big)).map(((p, q)) => min(p, q)).collect()
    best
print(if heads - 1 + k > n: -1 else: eat(t.root)[Head.Big][k]!)
