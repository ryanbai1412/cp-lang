// A fn or closure returning `?Int` that is lent a `mut` arg next to a fn
// value is a traced call on the vm, not a walk of the call.
struct Rope:
    xs: List<Int>
    fn path(mut self, i: Int, f: (mut List<Int>) -> ?Int) -> ?Int:
        if i < 0: None else: f(mut xs)

let n: Int = read()
var r = Rope([4, n, 6])
print(r.path(1, (mut v) => v.pop()) ?? -1, r.path(-1, (mut v) => v.pop()) ?? -1, r.xs)
var c = 3
let step = (mut k: Int, f: (Int) -> Int) => Some(f(k))
let pick = (mut k: Int, f: (Int) -> Int) => [k, n].get(f(k))
print(step(mut c, x => x + n) ?? -1, pick(mut c, x => x * 2) ?? -1, pick(mut c, x => n - 4) ?? -1, c)
