// expect: ok, output "2 6\n7 YES"
// rule: R04.2.18 R04.8.11
// A type that is `Discrete` but not a `Domain` keys an array as `Int` does: `[Step]T` is the list.
struct Step:
    k: Int
impl Discrete for Step:
    fn nth(k: Int) -> Step = Step(k)
    fn try_nth(k: Int) -> ?Step = Some(Step(k))
    @get fn ord(self) -> Int = k
let a: [Step]Int = [5, 6]
print(a.len, a[1])
print(Step.nth(7).ord, yes(Step.try_nth(-3) != None))
