// For each vertex: the sum of colours that occur most often in its subtree.
// Merge children's colour counts small into large; counts only grow, so the top count and the
// sum of colours reaching it update in O(1) per insertion.
let n: 1..=100_000 = read()
domain V = 1..=n
let colour: [V]1..=n = read()
let t = Rooted(Graph.undirected(read<[n - 1](V, V)>()), V.first)

struct Tally:
    var count: Map<Int, Int> = {}
    var top = 0   // the highest count
    var sum = 0   // the sum of colours with that count

    fn add(mut self, c: Int, times: Int):
        let x = (count[c] ?? 0) + times
        count[c] = x
        if x > top: (top, sum) = (x, c)
        elif x == top: sum += c

    fn absorb(mut self, mut other: Tally):
        if other.count.len > count.len: swap(mut self, mut other)
        for (c, k) in other.count: add(c, k)

var answer: [V]Int = 0
fn tally(v: V) -> Tally:
    var mine = Tally()
    mine.add(colour[v], 1)
    for u in t.children(v): mine.absorb(tally(u))
    answer[v] = mine.sum
    mine
tally(t.root)
print(answer)
