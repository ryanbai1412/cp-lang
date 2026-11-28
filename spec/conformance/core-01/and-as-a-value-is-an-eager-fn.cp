// expect: ok, output "NO YES\nNO YES\nran\nNO"
// rule: 01.5.7
// Bare `and` and `or` are std's eager fns `logical_and` and `logical_or`, so they pass as fn values; written `a and b` still short-circuits, and only a call of the fn runs its right side.
fn apply2(f: (Bool, Bool) -> Bool, a: Bool, b: Bool) -> Bool = f(a, b)
fn ran() -> Bool:
    print("ran")
    true
print(yes(apply2(and, true, false)), yes(apply2(or, false, true)))
print(yes(false and ran()), yes(true or ran()))
print(yes(apply2(and, false, ran())))
