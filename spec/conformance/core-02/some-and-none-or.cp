// expect: ok, output "true false"
// rule: 02.4.1
let a: ?Int = None
print(yes(a.is_none_or(x => x > 0), answers: ("true", "false")), yes(a.is_some_and(x => x > 0), answers: ("true", "false")))
