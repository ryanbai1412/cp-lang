// expect: ok, output "1"
// rule: 05.10.1
var cnt = 0
let bump = () => cnt += 1
bump()
print(cnt)
bump()
