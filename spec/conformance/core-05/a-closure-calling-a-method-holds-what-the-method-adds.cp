// expect: ok, output "Y Y Y"
// rule: 13.3.2
// A method that falls back to its own lambda (`self.g ?? (_ => true)`) adds that lambda to the
// field's set. A closure calling the method, returned from another method or made at the top
// level, holds the same set, so the closure's copy of the method is the one the call reaches.
struct Q<T>:
    g: ?(T -> Bool)
    fn z2(self, a: T) -> Bool = (self.g ?? (_ => true))(a)
    fn h1(self) -> List<T> -> Bool = s => self.z2(s[0])
    fn hs(self) -> List<List<T> -> Bool> = [s => self.z2(s[0])]
let q = Q<Int>(None)
let f = (s: List<Int>) => q.z2(s[0])
let yes = (b: Bool) => if b: "Y" else: "N"
print(yes(q.h1()([3])), yes(q.hs()[0]([3])), yes(f([3])))
