// expect: ok, output "4\n3\n3\n2"
// input: "5 4 0 1 1 2 0 2 3 4\n"
struct Dsu:
    p: List<Int>
    sz: List<Int>

    fn new(n: Int) -> Dsu:
        var p: List<Int> = []
        for i in 0..n: p.push(i)
        Dsu(p, List.filled(n, 1))

    fn find(mut self, x: Int) -> Int:
        var r = x
        while p[r] != r:
            p[r] = p[p[r]]
            r = p[r]
        r

    fn unite(mut self, a: Int, b: Int) -> Bool:
        var x = self.find(a)
        var y = self.find(b)
        if x == y: return false
        if sz[x] < sz[y]:
            let t = x
            x = y
            y = t
        p[y] = x
        sz[x] = sz[x] + sz[y]
        true

let n: Int = read()
let q: Int = read()
var d = Dsu.new(n)
var comps = n
for i in 0..q:
    let a: Int = read()
    let b: Int = read()
    if d.unite(a, b): comps -= 1
    print(comps)
