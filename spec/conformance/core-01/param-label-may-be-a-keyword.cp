// expect: ok, output "YES"
// rule: 01.10
// A parameter's outer label may be a reserved word: callers write `while:`, the body reads `ok`.
fn f(a: Int, while ok: Int -> Bool) -> Bool = ok(a)
print(yes(f(1, while: x => x > 0)))
