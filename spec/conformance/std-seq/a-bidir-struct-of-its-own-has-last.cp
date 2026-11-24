// expect: ok, output "2 1 | 1"
// rule: std.seq.last
// `last` is std's `@get fn last<S: Bidir, T>(self: S) -> ?T`: a struct of the file's own that is `Bidir<Int>` gets `?Int` from it, through its impl.
struct Two()
impl Seq<Int> for Two:
    fn iter(self) -> yields Int:
        yield 1
        yield 2
impl Bidir<Int> for Two:
    fn iter_rev(self) -> yields Int:
        yield 2
        yield 1
let x: Int = Two().last!
print(x, Two().first!, "|", Two().last! - Two().first!)
