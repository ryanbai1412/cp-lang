// expect: ok, output "7"
// engines: next
// rule: 06.4.12
// R06.4.12: a program's own `@coerce` fn is an implicit conversion where its result type is
// wanted, so `get(Cm(700))` runs `meters`. Frozen cpl prints 700: it never calls the fn and reads
// the `Cm` as a `Meters` field for field.
struct Cm:
    v: Int
struct Meters:
    v: Int
@coerce
fn meters(c: Cm) -> Meters = Meters(c.v / 100)
fn get(m: Meters) -> Int = m.v
print(get(Cm(700)))
