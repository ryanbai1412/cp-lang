// expect: ok, output "3\n1"
// rule: R05.2.2 R08.1.6
// A struct's derived `clone` copies each field, and a plain enum's `ord` is its variant's index.
struct V:
    k: Int
enum E:
    A
    B
let a = V(3)
print(a.clone().k)
print(E.B.ord)
