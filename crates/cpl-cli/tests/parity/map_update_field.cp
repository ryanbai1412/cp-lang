struct Reg:
    ids: Map<Str, Int>
    p: Int

    fn add(mut self, k: Str):
        ids.update(k, 10, v => v + 2)

    fn nest(mut self) -> Int:
        self.add("n")
        let seen = () => ids["n"] + p
        self.add("n")
        p += 100
        seen()

struct S:
    m: Map<Str, Int>
    r: Reg

fn deep(mut m: Map<Str, Int>, k: Str) -> Int:
    m.update(k, 10, v => v + 2)
    m.get_or_insert_with("z", () => m.len * 10)

var s = S(Map<Str, Int>(), Reg(Map<Str, Int>(), 0))
s.m["k"] = 10
s.m.update("k", 0, v => v + 2)
s.m.update("j", 2, v => v + 2)
print(s.m["k"], s.m["j"])
print(s.r.nest(), s.r.p)
print(deep(mut s.m, "k"), deep(mut s.r.ids, "q"))
print(s.m)
print(s.r.ids)
var all = [s]
all[0].m.update("k", 0, v => v * 2)
all[0].r.add("n")
print(all[0].m["k"], all[0].r.ids["n"], deep(mut all[0].m, "y"))
