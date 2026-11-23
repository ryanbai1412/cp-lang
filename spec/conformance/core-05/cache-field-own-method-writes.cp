// expect: ok, output "0 0"
// rule: 05.10.10
// The struct's own method writes its `@cache` field through a read `self`, even on a `let`.
domain V = 0..4
struct Uf:
    @cache parent: [V]V
    fn find(self, x: V) -> V:
        let p = self.parent[x]
        if p == x: return x
        let r = self.find(p)
        self.parent[x] = r
        r
let u = Uf(parent: [V(0), V(0), V(1), V(2)])
print(Int(u.find(V(3))), Int(u.parent[V(3)]))
