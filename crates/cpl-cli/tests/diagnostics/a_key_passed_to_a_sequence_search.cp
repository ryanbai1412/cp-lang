var m = Map<Str, Int>()
m["a"] = 1
print(m.find("a") ?? 0)
print(m.find(1))
let s = Set<Int>([1, 5])
if s.find(3):
    print(1)
print(s.find(x => x > 1) ?? 0)
