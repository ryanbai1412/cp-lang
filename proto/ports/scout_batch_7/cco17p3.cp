// CCO 2017 #3 Vera and Modern Art: an endless grid of zeros. A stroke
// x y v adds v to the cells (x + i a, y + j b) for all i, j >= 0, where a
// is the largest power of two not above x and b the same for y. After n
// strokes, print the value of each of q cells.
//
// Write x as a leading 1 followed by k low bits. The stroke reaches row r
// exactly when r has the same k low bits and at least k + 1 bits. So read
// a number from its lowest bit upwards: the strokes that reach r are those
// whose x is a node on the path that r spells, and the same holds for y
// and c. That is a binary trie over x in which every node owns a trie over
// y. Coordinates go up to 10^18, so both are built only where a stroke
// went: nodes live in one list and are named by their position in it.
struct Node:
    kid: [0..2]?Int  // by the next bit
    total: Int       // in a y trie: what the strokes ending here add
    ys: ?Int         // in the x trie: the root of the y trie of the strokes ending here

var nodes: List<Node> = []
fn fresh() -> Int:
    nodes.push(Node([0..2] _ => none, 0, none))
    nodes.len - 1

// The node that x spells below root, made on the way if it is missing.
fn reach(root: Int, x: Int) -> Int:
    var (o, rest) = (root, x)
    while rest > 1:
        nodes[o].kid[rest % 2] ??= fresh()
        o = nodes[o].kid[rest % 2]!
        rest /= 2
    o

// The y-trie roots that exist on the path x spells below root.
fn along_ys(root: Int, x: Int) -> yields Int:
    var (o, rest) = (root, x)
    if let ys = nodes[o].ys: yield ys
    while rest > 1:
        if let next = nodes[o].kid[rest % 2]:
            o = next
            if let ys = nodes[o].ys: yield ys
            rest /= 2
        else:
            return

// The sum of totals along the path y spells below a y-trie root.
fn sum_along(root: Int, y: Int) -> Int:
    var (o, rest, total) = (root, y, nodes[root].total)
    while rest > 1:
        if let next = nodes[o].kid[rest % 2]:
            o = next
            total += nodes[o].total
            rest /= 2
        else:
            break
    total

let (n, q): (Int, Int) = read()
let xs = fresh()
for _ in 0..n:
    let (x, y, v): (Int, Int, Int) = read()
    let col = reach(xs, x)
    nodes[col].ys ??= fresh()
    nodes[reach(nodes[col].ys!, y)].total += v

for _ in 0..q:
    let (r, c): (Int, Int) = read()
    print(sum(sum_along(ys, c) for ys in along_ys(xs, r)))
