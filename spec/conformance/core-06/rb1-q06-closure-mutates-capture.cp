// expect: ok, output "1\n2"
// rule: 06.2.1
var cnt = 0
let inc = () => cnt += 1
inc()
print(cnt)
inc()
print(cnt)
