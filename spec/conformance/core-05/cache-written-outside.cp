// expect: L0609 at 11
// rule: 05.10.10
struct Dsu:
    @cache parent: List<Int>
    pub fn find(self, x: Int) -> Int:
        let p = self.parent[x]
        if p == x: return x
        let r = self.find(p)
        self.parent[x] = r
        r

var d = Dsu(parent: [0, 0, 1])
d.parent[2] = 2
