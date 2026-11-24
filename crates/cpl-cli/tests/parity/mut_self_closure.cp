struct Reg:
    ids: Map<Str, Int>
    p: Int

    fn id(mut self, name: Str) -> Int:
        ids.update(name, -1, v => if v < 0: ids.len else: v)
        ids[name]

    fn id2(mut self, name: Str) -> Int:
        self.ids.update(name, -1, v => if v < 0: self.ids.len else: v)
        self.ids[name]

fn idm(mut ids: Map<Str, Int>, name: Str) -> Int:
    ids.update(name, -1, v => if v < 0: ids.len else: v)
    ids[name]

var r = Reg(Map<Str, Int>(), 0)
print("a", r.id("a"), "b", r.id("b"), "a", r.id("a"), "c", r.id2("c"))
var m = Map<Str, Int>()
print(idm(mut m, "a"), idm(mut m, "b"), idm(mut m, "a"), idm(mut m, "c"))
var s = (r, 7)
print(s.0.id("d"), s.0.id2("e"), idm(mut s.0.ids, "f"), s.0.ids.len)
