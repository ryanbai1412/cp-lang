// expect: ok, output "3 -4"
// rule: 02.7.1
// Variants live in their enum's scope, so `Set` and `Add` are fine variant names.
enum Q = Set(Int) | Add(Int)
fn val(q: Q) -> Int = match q:
    Set(x) => x
    Add(x) => -x
print(val(Set(3)), val(Add(4)))
