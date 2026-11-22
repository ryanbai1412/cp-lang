use std.derive.read_fields

@allow(unread_field)
struct Edge(u: Int, v: Int, w: Int)
@allow(unread_field)
struct Item(name: Str, at: Edge, ok: Bool)

let e = read_fields<Edge>()
let f = read_fields<Edge>()
let it = Item(read() for _ in fields(Item))
let sum = Edge(x + y for (x, y) in fields(e, f))
print(e, f, sum)
print(it.name, it.at, it.ok)
print(Edge(-x for x in fields(e)) == Edge(-1, -2, -3))
