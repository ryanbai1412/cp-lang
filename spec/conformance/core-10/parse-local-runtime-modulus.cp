// expect: ok, output "2 2 2"
// rule: 10.2.4
// A `type Z = Mod(p)` inside a fn parses and reads by `Mod`'s own input format.
fn under(p: Int, digits: List<Str>) -> List<Int>:
    type Z = Mod(p)
    [s.parse<Z>()!.v for s in digits]
fn first(p: Int) -> Int:
    type Z = Mod(p)
    "-12".parse<Z>()!.v
print(under(7, ["-12", "100000000000000000000"]), first(7))
