// expect: E0605 at 5
// rule: 04.2.26
// R04.2.26: an `impl Index` writes `count`, `ord` and `nth`; the pre-0.3 names `K`, `to_idx` and `from_idx` are errors that name the new one.
struct Id(i: Int)

impl Index for Id:
    let K = 3
    @get fn ord(self) -> Int = i
    fn nth(k: Int) -> Id = Id(k)
