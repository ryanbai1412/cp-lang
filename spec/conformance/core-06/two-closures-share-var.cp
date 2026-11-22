// expect: ok, output "1"
// rule: 06.2.1 05.10.3
var c = 0
let inc = () => c += 1
let dec = () => c -= 2
inc()
inc()
dec()
inc()
print(c)
