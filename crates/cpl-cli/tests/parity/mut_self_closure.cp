struct Reg:
    ids: Map<Str, Int>
    p: Int

    fn id(mut self, name: Str) -> Int:
        ids.get_or_insert_with(name, () => ids.len)

    fn id2(mut self, name: Str) -> Int:
        self.ids.get_or_insert_with(name, () => self.ids.len)

fn idm(mut ids: Map<Str, Int>, name: Str) -> Int:
    ids.get_or_insert_with(name, () => ids.len)

var r = Reg(Map<Str, Int>(), 0)
print("a", r.id("a"), "b", r.id("b"), "a", r.id("a"), "c", r.id2("c"))
var m = Map<Str, Int>()
print(idm(mut m, "a"), idm(mut m, "b"), idm(mut m, "a"), idm(mut m, "c"))
var s = (r, 7)
print(s.0.id("d"), s.0.id2("e"), idm(mut s.0.ids, "f"), s.0.ids.len)
