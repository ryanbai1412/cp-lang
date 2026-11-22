// expect: ok, output "2"
// rule: 05.10.3 06.2.1
var cnt = 0
let inc = () => cnt += 1
inc()
inc()
print(cnt)
