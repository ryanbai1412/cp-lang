// expect: R0001
// rule: R04.2.27
// `D.nth(k)` outside `0..D.count` traps before the impl's `nth` sees `k`.
struct Id(i: Int)
impl Index for Id:
    let count = 3
    @get fn ord(self) -> Int = i
    fn nth(k: Int) -> Id = Id(k)
fn pick<D: Domain>(k: Int) -> D = D.nth(k) // <- here
let k = 3
let p: Id = pick(k)
print(p.i)
