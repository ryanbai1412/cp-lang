let n: Int = read()
var x: U64 = 0
for i in 0..n:
    x = splitmix64(x + U64(i))
    print(x)
print(splitmix64(0), splitmix64(U64(-1)))
