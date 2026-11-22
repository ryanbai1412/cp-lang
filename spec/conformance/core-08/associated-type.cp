// expect: ok, output "8"
// rule: 08.2.7
trait Box:
    type Item
    fn get(self) -> Self.Item
struct IntBox: v: Int
impl Box for IntBox:
    type Item = Int
    fn get(self) -> Int = self.v
fn both<B: Box>(b: B) -> (B.Item, B.Item) = (b.get(), b.get())
let (x, y) = both(IntBox(4))
print(x + y)
