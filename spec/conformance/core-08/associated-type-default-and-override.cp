// expect: ok, output "4 9"
// rule: 08.1.9
// An impl that leaves out a defaulted associated type gets the default, and one that writes `type Item = T` overrides it (decision 469).
trait Box:
    type Item = Int
    fn get(self) -> Self.Item
struct IntBox: v: Int
impl Box for IntBox:
    fn get(self) -> Int = self.v
struct ListBox: v: Int
impl Box for ListBox:
    type Item = List<Int>
    fn get(self) -> List<Int> = [self.v]
print(IntBox(4).get(), ListBox(9).get())
