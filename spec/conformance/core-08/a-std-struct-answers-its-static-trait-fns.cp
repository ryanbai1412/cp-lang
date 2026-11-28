// expect: ok, output "YES"
// rule: 08.2.5
// A std struct's name answers the static fns of the traits it implements, as a
// user struct's does: `Str.default()` is std's `impl Default for Str`.
trait Z:
    fn z() -> Self
impl Z for Str:
    fn z() -> Str = "z"
let s: Str = Str.default()
print(yes(s.len == 0 and Str.z() == "z"))
