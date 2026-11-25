// expect: E0405
// (spec) R02.6.1 (today accepts bare `hits += 1` on a fixed field: compiler bug)
// A lender that lends a field directly (no copy-in/copy-out).
struct C:
    v: Int
    hits: Int
    fn slot(mut self) -> lends mut Int:
        hits += 1  // <- here
        lend mut v
        hits += 100
fn go() -> Int:
    var c = C(1, 0)
    c.slot() = 5
    c.slot() += 2
    c.v * 1000 + c.hits
print(go())
