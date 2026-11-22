// expect: ok, output "42"
// rule: 08.1.1
// Calling a generic type alias without explicit type arguments infers its
// type parameters from the expected type or constructor arguments.
struct Boxed<T>:
    val: T

type AliasBox<T> = Boxed<T>

let b: AliasBox<Int> = AliasBox(42)
print(b.val)
