// expect: E0602 at 12
// rule: 05.10.11
struct Dsu:
    @cache parent: List<Int>
    pub fn find(self, x: Int) -> Int:
        let p = self.parent[x]
        if p == x: return x
        let r = self.find(p)
        self.parent[x] = r
        r

    pub fn roots(self) -> Int:
        var c = 0
        for p in self.parent:
            if self.find(p) == p: c += 1
        c
