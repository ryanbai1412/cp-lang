// expect: R0001 at 10
// rule: 04.2.27
// R04.2.27: `ord` is the impl's own code, so indexing `[T]E` checks that `x.ord` lands in `0..T.count`, as `V.at(k)` does.
struct Id(i: Int)

impl Index for Id:
    let count = 3
    @get fn ord(self) -> Int = i
    fn nth(k: Int) -> Id = Id(k)

var t: [Id]Int = 0
t[Id(7)] = 1
print(t[Id(1)])
