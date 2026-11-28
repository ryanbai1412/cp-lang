// expect: E0202
// rule: 11.6.1
// std's `Lawful` fills the `lawful` role, so a program's own trait marked `@role(lawful)` is a
// second filler of it.
@role(lawful) // <- here
trait MyLaws<S>:
    @get fn laws(self) -> List<(name: Str, sizes: S -> List<Int>, holds: (S, List<Int>) -> Bool)>
struct M:
    x: Int
    law small(a: M) = a.x < 2
print(M(0).laws.len)
