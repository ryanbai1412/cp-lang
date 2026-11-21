fn twice(n: Int, f: (Int) -> Int) -> Int = f(f(n))
print(twice(1) { x => x + 1 })
