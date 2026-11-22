// section 2: `if let a = x and let b = y and a > 0:` mixes optional bindings and Bool guards
var x: ?Int = 3
var y: ?Int = 4
if let a = x and let b = y and a > 0: print(a + b)
if let a = x and let b = y and a > 5: print(0)
else: print(-1)
