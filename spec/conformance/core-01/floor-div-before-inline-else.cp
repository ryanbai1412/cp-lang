// expect: E0107 at 3
// rule: 01.1.2
// has: "already rounds down"
// `x // 2` before an inline `else:` is a C++ habit, not a comment that hides the `else`: the fix is `/`.
var x = 6
x = if x % 2 == 0: x // 2 else: 3 * x + 1
print(x)
