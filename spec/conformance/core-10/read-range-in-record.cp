// expect: R0502 (checked)
// engines: next
// rule: 10.2.2 08.4.5
// input: "4 3\n4 9\n"
// A record's `1..=3` field, read in its `Read` impl, goes through the range type's own `Read`:
// outside it the record is `None` from a string and traps from standard input.
struct P:
    a: Int
    b: 1..=3
impl Read for P:
    fn read(mut input: Input) -> P = P(input.read(), input.read()) // <- here
let p = read<P>(from: "5 2")!
assert p.a == 5 and p.b == 2
assert read<P>(from: "5 9") == None
assert read<List<P>>(from: "2 1 1 2 4") == None
assert read<P>().b == 3
print(read<P>().b)
