// expect: ok, output "1\n0"
// rule: R08.1.6
// A struct with no `Eq` of its own compares field by field.
struct V:
    k: Int
print(if V(3) == V(3): 1 else: 0)
print(if V(3) == V(4): 1 else: 0)
