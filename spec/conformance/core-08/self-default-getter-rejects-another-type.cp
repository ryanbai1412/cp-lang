// expect: E0606
// rule: 08.2.5
// A struct type argument bound by a `= self` getter trait must be the getter's type or a record
// with that field: `Str` is not a `Tagged<Int>`.
trait Tagged<T>:
    @get fn tag(self) -> T = self
struct Hold<T, A: Tagged<T>>:
    a: A
let h: Hold<Int, Str> = Hold(a: "x")  // <- here
