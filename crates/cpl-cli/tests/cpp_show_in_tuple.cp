// A part a `@cpp` `show` fn prints is C++ only, as a direct call is.
struct P: x: Int
impl Show for P:
    fn show(p) -> Str = @cpp("return \"P\";")
let t = (P(1), 2)
print(t)
