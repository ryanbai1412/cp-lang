// expect: E0203
// rule: 08.1.7
// A named struct is nominal: a trait's default methods are its only through an `impl`, even when its fields meet the trait.
trait Square:
    @get fn w(self) -> Int
    fn area(self) -> Int = self.w * self.w
struct P:
    w: Int
print(P(3).area()) // <- here
