// expect: E0604
// rule: 05.10.9
struct Dsu:
    @cache parent: List<Int>
    pub fn find(self, x: Int) -> Int:
        let p = self.parent[x]
        if p == x: return x
        let r = self.find(p)
        self.parent[x] = r
        r

    pub fn parents(self) -> lends List<Int> = self.parent // <- here
