// cp: 0.1 83a658e+dirty
let n: 1..=9 = read()
domain V = 1..=n

struct Edge:
    u, v: V
    w: 0..=1000000000

    fn other(self, x: V) -> V:
        if x == u: v else: u
    fn relax(mut self, d: Int):
        w = min(w, d)
    fn make(a: V, b: V) -> Edge:
        Edge(a, b, 1000)

var e = Edge.make(V.first!, V.last!)
e.relax(7)
e.relax(50)
print(e.other(V.first!), e.w)
