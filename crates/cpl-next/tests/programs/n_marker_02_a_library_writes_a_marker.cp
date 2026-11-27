// expect: E0202
// rule: R06.4.12
// A library writes a marker like core does; a second `@operator("+")` with core's method is a
// duplicate, reported at the program's own.
// file: cp.toml "[libs]\nkit = \"kit\"\n"
// file: kit/m.cp "pub fn one() -> Int = 1\n"
use kit.m.{one}
@operator("+") // <- here
trait Plus:
    fn add(self, o: Self) -> Self
print(one())
