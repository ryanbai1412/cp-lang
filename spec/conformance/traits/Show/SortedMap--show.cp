// expect: ok, output "1=a, 3=c"
var m: SortedMap<Int, Str> = SortedMap()
m.set(3, "c")
m.set(1, "a")
print(m)
