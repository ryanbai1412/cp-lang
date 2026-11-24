var m = Map<Str, Int>()
m["a"] = 1
print(m.find("a") ?? 0)  // <- here
