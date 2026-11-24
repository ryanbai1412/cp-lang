// expect: ok, output "3 4\n5"
// rule: 08.1.9
// A trait's default fn sees the trait's parameters as the impl's trait args, and `Self.Out` as the type the impl chooses or else the trait's default (decision 469).
trait Wrap<T>:
    type Out = ?T
    fn base(self, x: T) -> T
    fn wrap(self, x: T) -> Self.Out = Some(self.base(x))
    fn twice(self, x: T) -> List<T> = [self.base(x), self.base(x)]
struct Id
impl Wrap<Int> for Id:
    fn base(self, x: Int) -> Int = x
struct Inc
impl Wrap<Int> for Inc:
    type Out = Int
    fn base(self, x: Int) -> Int = x + 1
    fn wrap(self, x: Int) -> Int = self.base(x)
let o: ?Int = Id().wrap(3)
print(o ?? -1, Inc().wrap(3))
print(Id().twice(2).sum() + Inc().twice(0).len - 1)
