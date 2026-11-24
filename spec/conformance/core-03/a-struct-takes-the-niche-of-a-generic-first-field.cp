// expect: ok, output "none\nsome YES -1\nsome NO 3\n1 YES YES 0\n3 NO YES 9\nnone\n5"
// rule: 03.2.2
// A struct's first field with a spare value gives `?S` its `None`, through a
// generic instance (`Id<Int>`) and its own `?`: `Some(N())`, whose `a` is
// `None`, stays apart from `None`.
use std.marks.{Id}
struct N:
    var a: ?Id<Int> = None
    var b: ?(0..=2_000_000_000) = None
struct Node:
    var key: (at: Int, cnt: Int)
    var l: ?Id<Int> = None
    var r: ?Id<Int> = None
    var stamp: 0..=2_000_000_000 = 0
var ns: List<?N> = [None, Some(N()), Some(N(a: Some(Id(7)), b: Some(3)))]
for n in ns:
    if let x = n: print("some", yes(x.a == None), x.b ?? -1)
    else: print("none")
var nodes: List<?Node> = [Some(Node(key: (at: 1, cnt: 2))), Some(Node(key: (at: 3, cnt: 4), l: Some(Id(0)), stamp: 9)), None]
for o in nodes:
    if let v = o: print(v.key.at, yes(v.l == None), yes(v.r == None), v.stamp)
    else: print("none")
nodes[0]!.l = Some(Id(5))
print(nodes[0]!.l!.raw)
