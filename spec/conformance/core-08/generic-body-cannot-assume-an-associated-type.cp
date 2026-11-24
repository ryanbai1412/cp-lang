// expect: E0302
// rule: 08.1.9 08.2.2
// A generic body sees only `B.Item`, not the one impl that chose `Int`, so returning it as an `Int` is an error at the definition.
trait Box:
    type Item
    fn get(self) -> Self.Item
struct IntBox: v: Int
impl Box for IntBox:
    type Item = Int
    fn get(self) -> Int = self.v
fn as_int<B: Box>(b: B) -> Int = b.get() // <- here
print(as_int(IntBox(4)))
