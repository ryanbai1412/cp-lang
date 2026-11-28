// expect: ok, output "3"
// rule: R08.1.6
// A struct with no `Show` of its own gets it from its fields.
struct V:
    k: Int
print(V(3))
