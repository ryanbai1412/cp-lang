domain V = 1..=6
let k: Int = read()
for v in V.rev by 2: print(v)
print("before")
for v in V.rev by k: print(v)
print("unreached")
