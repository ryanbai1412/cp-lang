// `for f in fields(x)`: the body once per field, in declaration order (decision 28.1).
struct Edge(u: Int, v: Int, w: Str)

fn show_all<T>(x: T):
    for f in fields(x):
        print(f)

fn same<T>(a: T, b: T) -> Int:
    var n = 0
    for (x, y) in fields(a, b):
        if x == y:
            n += 1
    n

let e: Edge = read()
for f in fields(e):
    print(f)
show_all(e)
show_all((5, 'c', true))
print(same(e, Edge(e.u, 0, e.w)))
print(same((1, 2), (1, 2)))
