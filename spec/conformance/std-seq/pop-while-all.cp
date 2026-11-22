// expect: ok, output "3 0"
var s = [1, 2, 3]
let out = s.pop_while(x => x > 0)
print(out.len, s.len)
