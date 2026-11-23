// expect: ok, output "fast fast loop"
// `if T: Tr` holds for an instance of a generic struct with `impl Tr for S<P>`, and a branch the type
// can't take is never lowered, so it may call the trait's methods.
trait Fast:
    fn tag() -> Str
struct Box<const P: Int>(v: Int)
impl Fast for Box<P>:
    fn tag() -> Str = "fast"
struct Leaf(v: Int)
impl Fast for Leaf:
    fn tag() -> Str = "fast"
struct Plain(v: Int)
fn pick<T>(x: T) -> Str:
    if T: Fast: return T.tag()
    "loop"
print(pick(Box<3>(1)), pick(Leaf(2)), pick(Plain(3)))
