// expect: ok, output "0 0"
// rule: 05.10.14 06.4.4
struct Dsu:
    @cache parent: List<Int>
    pub fn find(self, x: Int) -> Int:
        let p = self.parent[x]
        if p == x: return x
        let r = self.find(p)
        self.parent[x] = r
        r

let d = Dsu(parent: [0, 0, 1])
print(d.find(2), d.parent[2])
