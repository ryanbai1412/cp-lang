// expect: ok, output "2"
var cnt = 0
let inc = () => cnt += 1
inc()
inc()
print(cnt)
