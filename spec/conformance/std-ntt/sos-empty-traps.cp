// expect: R0401 (checked)
var a: List<Int> = []
sos(mut a) // <- here
print(a.len)
