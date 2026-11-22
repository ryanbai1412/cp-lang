// expect: ok, output "2"
// rule: 05.10.8
domain V = 0..3
pub struct Dsu<V>:
    @cache parent: [V]V
    size: [V]Int

    pub fn find(self, x: V) -> V:
        let p = self.parent[x]
        if p == x: return x
        let r = self.find(p)
        self.parent[x] = r
        r
let d = Dsu(parent: [V] v => v, size: [V] _ => 1)
print(d.find(2))
