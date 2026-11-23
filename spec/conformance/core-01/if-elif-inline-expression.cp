// expect: ok, output "mid"
// rule: 01.7.1
// The inline `if c: a elif d: b else: e` is an expression (rebuild 7, H-IF-EXPR-ELIF).
let n = 5
let s = if n < 3: "lo" elif n < 7: "mid" else: "hi"
print(s)
