use std.flow.{FlowGraph}
use std.dsu.{Dsu}

// §2.16: Algebraic Index derivation — payload enums and tuples/records of index types.
domain Row = 0..2
domain Col = 0..3

enum Node:
    Src
    Snk
    Cell(Row, Col)

print(Node.count)
var vals: [Node]Int = 0
for v in Node:
    vals[v] = match v:
        Src => 100
        Snk => 200
        Cell(r, c) => r.ord * 10 + c.ord
print(vals[Node.Src], vals[Node.Snk], vals[Node.Cell(Row(1), Col(2))])

var dsu = Dsu<Node>()
dsu.unite(Node.Src, Node.Cell(Row(0), Col(0)))
dsu.unite(Node.Cell(Row(0), Col(0)), Node.Cell(Row(1), Col(2)))
print(dsu.same(Node.Src, Node.Cell(Row(1), Col(2))), dsu.same(Node.Src, Node.Snk))

var fg = FlowGraph<Node>()
fg.add(Node.Src, Node.Cell(Row(0), Col(0)), 4)
fg.add(Node.Cell(Row(0), Col(0)), Node.Snk, 3)
print(fg.max_flow(Node.Src, Node.Snk))

// Tuples and records of index types derive Index (§2.16).
var fg_tup = FlowGraph<(Row, Col)>()
fg_tup.add((Row(0), Col(0)), (Row(0), Col(1)), 5)
fg_tup.add((Row(0), Col(1)), (Row(1), Col(2)), 3)
print(fg_tup.max_flow((Row(0), Col(0)), (Row(1), Col(2))))

var fg_rec = FlowGraph<(r: Row, c: Col)>()
fg_rec.add((r: Row(0), c: Col(0)), (r: Row(1), c: Col(2)), 7)
print(fg_rec.max_flow((r: Row(0), c: Col(0)), (r: Row(1), c: Col(2))))

var grid: [(Row, Col)]Int = 0
grid[(Row(1), Col(2))] = 42
print(grid[(Row(1), Col(2))])

// Sum enum with Bool + fieldless enum payloads.
enum Dir:
    Up
    Down

enum State:
    Idle
    Moving(Row, Dir, Bool)

print(State.count)
var s_arr: [State]Int = 0
s_arr[State.Idle] = 1
s_arr[State.Moving(Row(1), Dir.Down, true)] = 99
print(s_arr[State.Idle], s_arr[State.Moving(Row(1), Dir.Down, true)])
