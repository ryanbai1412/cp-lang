// expect: E0606
// rule: 08.1.9
// An impl must choose every associated type its trait declares, even when its methods spell the type out.
trait Box:
    type Item
    fn get(self) -> Self.Item
struct IntBox: v: Int
impl Box for IntBox: // <- here
    fn get(self) -> Int = self.v
print(IntBox(4).get())
