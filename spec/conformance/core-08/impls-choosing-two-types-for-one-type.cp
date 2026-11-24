// expect: E0202 at 8
// rule: 08.1.9 08.1.2
// A blanket impl and a specific one both apply to `List<Int>`, so it would have two choices of `Out`; one impl per type.
trait Size:
    type Out
    fn size(self) -> Self.Out
impl<T> Size for List<T>:
    type Out = Int
    fn size(self) -> Int = self.len
impl Size for List<Int>:
    type Out = Str
    fn size(self) -> Str = "{self.len}"
print([1, 2].size())
