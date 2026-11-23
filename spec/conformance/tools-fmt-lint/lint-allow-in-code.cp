// expect: ok
// rule: lint.2
// tool: lint
// lacks: "L0201"
with lint(var_never_reassigned: allow)
var x = 1
print(x)
