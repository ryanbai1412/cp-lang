// expect: ok, output "21"
// input: "1\n"
// A stored operation wrapped by a fn of the struct; the two branches meet in a new, larger set.
struct Monoid:
    id: Int
    op: (Int, Int) -> Int

    fn dual(self) -> Monoid = Monoid(id, (a, b) => op(b, a))
let n: Int = read()
let base = Monoid(0, (a, b) => a * 10 + b)
let m = if n % 2 == 1: base.dual() else: base
print(m.op(1, 2))
