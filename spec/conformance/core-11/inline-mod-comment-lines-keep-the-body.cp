// expect: ok, output "3"
// rule: 11.3.1
// A comment-only line in an inline `mod`'s body, at any indent, is skipped as in any block: it neither sets the body's indent nor becomes code.
mod m:
// m
    pub fn f() -> Int = 1
  // g()
    pub fn g() -> Int = f() + 2
print(m.g())
