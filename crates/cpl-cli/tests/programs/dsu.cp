// Union-find as a struct with methods, driving Kruskal's algorithm.
struct Dsu:
    parent: List<Int>
    size: List<Int>

    fn make(n: Int) -> Dsu:
        var parent: List<Int> = []
        var size: List<Int> = []
        for i in 0..n:
            parent.push(i)
            size.push(1)
        Dsu(parent: parent, size: size)

    fn find(mut self, x: Int) -> Int:
        var root = x
        while self.parent[root] != root:
            root = self.parent[root]
        var at = x
        while self.parent[at] != root:
            let next = self.parent[at]
            self.parent[at] = root
            at = next
        root

    fn unite(mut self, a: Int, b: Int) -> Bool:
        var x = self.find(a)
        var y = self.find(b)
        if x == y:
            return false
        if self.size[x] < self.size[y]:
            let t = x
            x = y
            y = t
        self.parent[y] = x
        self.size[x] += self.size[y]
        true

struct Edge:
    w: Int
    u: Int
    v: Int

let n: Int = read()
let m: Int = read()
var edges: List<Edge> = []
for i in 0..m:
    let u: Int = read()
    let v: Int = read()
    let w: Int = read()
    edges.push(Edge(w: w, u: u, v: v))
edges.sort(by: e => e.w)
var dsu = Dsu.make(n)
var cost = 0
var used: List<Edge> = []
for e in edges:
    if dsu.unite(e.u, e.v):
        cost += e.w
        used.push(e)
print(cost, used.len)
print(used)
print(dsu.find(0) == dsu.find(n - 1), dsu.size[dsu.find(0)])
