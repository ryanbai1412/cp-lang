// expect: E0602 at 11
// rule: 05.10.12
struct Dsu:
    @cache parent: List<Int>
    pub fn find(self, x: Int) -> Int:
        let p = self.parent[x]
        if p == x: return x
        let r = self.find(p)
        self.parent[x] = r
        r

    pub fn each(self, f: (Int) -> Int):
        for p in self.parent: f(p)
