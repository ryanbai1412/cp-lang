// expect: ok, output "2 1 2"
// rule: 07.5.10
// A `for` header's temporary lives for the whole loop; the C++ once freed it early (rebuild round 16 R16-TEMP-VIEW-UAF).
let a = [3, 1, 3, 2, 1]
var lens: List<Int> = []
for r in a.sorted().runs(): lens.push(r.len)
print(lens)
