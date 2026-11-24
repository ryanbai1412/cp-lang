// expect: ok, output "twice 9"
// rule: 08.1.9
trait Named:
    fn label(self) -> Str
fn twice(a: Int, b: Int) -> Int = 2 * a + b
impl Named for type_of(twice):
    fn label(self) -> Str = "twice"
fn tag<M: Named>(m: M) -> Str = m.label()
fn apply(f: (Int, Int) -> Int, x: Int) -> Int = f(x, x)
print(tag(twice), apply(twice, 3))
