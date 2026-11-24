struct Reg:
    var ids: Map<Str, Int>
    var p: Int

    fn id(mut self, name: Str) -> Int:
        ids.update(name, -1, v => if v < 0: ids.len else: v)
        ids[name]

    fn late(mut self) -> Int:
        let f = () => p
        p += 1
        f()

fn touch(mut r: Reg) -> Int:
    let before = () => r.p
    r.p += 5
    before() + r.late()

fn raise(mut x: Int, by: Int) -> Int:
    let seen = () => x
    x += by
    seen()

var arr = [Reg(Map<Str, Int>(), 0), Reg(Map<Str, Int>(), 0)]
print(arr[0].id("x"), arr[0].id("y"), arr[1].id("z"))
print(arr[1].late(), arr[1].p, touch(mut arr[0]), arr[0].p)
var grid: [0..2, 0..2]Reg = Reg(Map<Str, Int>(), 0)
print(grid[1, 0].id("a"), grid[1, 0].id("b"), grid[0, 1].id("c"), grid[1, 0].late())
var by_key = Map<Int, Reg>()
by_key[3] = Reg(Map<Str, Int>(), 0)
print(by_key[3].id("a"), by_key[3].id("b"), by_key[3].late())
var xs = [1, 2, 3]
print(raise(mut xs[1], 10), raise(mut arr[0].p, 100), xs, arr[0].p)
