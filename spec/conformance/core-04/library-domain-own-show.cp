// expect: ok, output "0<1\n0<2\n1<2\n3"
// rule: 04.2.28
// R04.2.28: a library domain prints through its own `Show`, not its ordinal.
struct Two(a: Int, b: Int)

impl Index for Two:
    let count = 3
    @get fn ord(self) -> Int = a + b - 1
    fn nth(k: Int) -> Two = if k < 2: Two(0, k + 1) else: Two(1, 2)

impl Show for Two:
    fn show(t) = "{t.a}<{t.b}"

for t in Two: print(t)
print(Two.count)
