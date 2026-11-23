// expect: ok, output "7"
// rule: 01.1.2
// `//` after code on a line with no open `if` starts a comment, even when the comment says `else:`.
var p: ?Int = None  // WISH: let r = f(p) else: ...
print(p ?? 7)
