// expect: ok, output "3 0 7"
// rule: 11.6.1
// A program's own `Optional` and `Show` shadow those names; `?Int`, `??` and `print` still use
// the declarations std marks with `@role`.
struct Optional(v: Int)
trait Show:
    fn shown(self) -> Int
let x: ?Int = Some(3)
let y: ?Int = None
print(x ?? 0, y ?? 0, Optional(7).v)
