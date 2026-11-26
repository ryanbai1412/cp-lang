// expect: ok, output "1 one 0 zero\n7 seven"
// rule: R08.1.2
// A type with impls of one trait at different arguments: the bound's arguments pick the impl.
trait Conv<T>:
    fn conv(self) -> T
impl Conv<Int> for Bool:
    fn conv(self) -> Int = if self: 1 else: 0
impl Conv<Str> for Bool:
    fn conv(self) -> Str = if self: "one" else: "zero"
fn num<X: Conv<Int>>(x: X) -> Int = x.conv()
fn word<X: Conv<Str>>(x: X) -> Str = x.conv()
fn both<X: Conv<Int> + Conv<Str>>(x: X) -> Int = num(x)
print(num(true), word(true), both(false), word(false))
struct Seven:
    n: Int
impl Conv<Int> for Seven:
    fn conv(self) -> Int = n
impl Conv<Str> for Seven:
    fn conv(self) -> Str = "seven"
print(num(Seven(7)), word(Seven(7)))
