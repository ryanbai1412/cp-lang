// expect: ok, output "8 7"
// `init:` by name sets what a missing key reads; `push` adds 1 on top of it.
var c: Counter<Int, Int> = Counter(init: 7)
c.push(1)
print(c[1], c[2])
