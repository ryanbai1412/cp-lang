// expect: E0302
// rule: 06.4.12
// R06.4.12: a `@coerce` fn applies only where its parameter's type is a struct or an enum, so
// an `Int` is never filled into a unit newtype implicitly: `get(700)` is a mismatch at the use,
// on frozen cpl and next alike.
struct Meters:
    v: Int
@coerce
fn meters(c: Int) -> Meters = Meters(c / 100)
fn get(m: Meters) -> Int = m.v
print(get(700)) // <- here
