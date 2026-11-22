// A struct field that is a row of a count written as a number is kept
// inside the struct in the C++: indexing, length, copies, whole-row
// assignment and comparison read the same as in `cpl run`.
struct Node:
    kid: [0..2]?Int
    cnt: [1..=3]Int
    total: Int

struct Pt:
    at: [0..3]Int

fn sum3(a: [1..=3]Int) -> Int: a[1] + a[2] + a[3]

var nodes: List<Node> = []
let n: Int = read()
for i in 0..n:
    var kid: [0..2]?Int = none
    var cnt: [1..=3]Int = i
    nodes.push(Node(kid, cnt, i))
nodes[0].kid[1] = 5
nodes[0].cnt[3] += 2
nodes[1].kid[n % 2] ??= 7
let c = nodes[0]
print(c.kid[1] ?? -1, c.kid[0] ?? -1, c.cnt, c.cnt.len, c.kid.len)
print(nodes[1].kid[n % 2]!, sum3(nodes[2].cnt))
var copy = nodes[2]
copy.cnt[1] = 100
print(copy.cnt, nodes[2].cnt, copy.cnt == nodes[2].cnt, nodes[1].cnt == nodes[1].cnt)
let whole = nodes[2].cnt
nodes[2].cnt = copy.cnt
print(whole, nodes[2].cnt)
for x in nodes[2].cnt: print(x)
for (i, x) in nodes[0].kid.items: print(i, x ?? -1)
var grow: [1..=3]Int = 9
nodes[1].cnt = grow
print(sum(nodes[1].cnt), max(nodes[1].cnt), [x * 2 for x in nodes[0].cnt])
let lit: [0..3]Int = [1, 2, 3]
var a = Pt(lit)
var b = Pt(lit)
b.at[2] = 0
print(a == b, b.at, a)
var pts = [a, b]
pts.sort(by: p => p.at[2])
print(pts)

// A row that is sorted, filled or handed to a fn as `mut` stays a list.
struct Bag:
    held: [0..4]Int
    tag: [0..2]Int

fn bump(mut row: [0..2]Int):
    row[0] += 10

let held: [0..4]Int = [4, 3, 2, 1]
var tag: [0..2]Int = 0
var bag = Bag(held, tag)
bag.held.sort()
bump(mut bag.tag)
print(bag.held, bag.tag, bag.held.len)
bag.held.fill(7)
print(bag)
