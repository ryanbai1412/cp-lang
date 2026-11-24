// expect: ok, output "4\n9"
// rule: 01.7.1
// The value of `if let` / `elif let` runs to the `:`, `??` included; only `and` ends it.
let a: ?Int = None
let b: ?Int = Some(4)
if a == Some(1):
    print(1)
elif let c = a ?? b:
    print(c)
if let d = a ?? b and d > 5:
    print(d)
elif let e = a ?? Some(9):
    print(e)
