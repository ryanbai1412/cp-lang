// expect: ok, output "1 0"
// rule: 03.3.1
let a: ?Int = None
let b: ?Int = Some(-1)
print(if a == None: 1 else: 0, if b == None: 1 else: 0)
