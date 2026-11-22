type Z = Mod<1e9 + 7>

var sum: Z = 0
for a in 0..3, b in 0..=a if a + b > 0:
    sum += a * 10 + b
print(sum)

let found: ?Int = some(42)
guard let v = found else: return print(-1)
let empty: ?Int = none
guard let _ = empty else: return print(v)
print(999)
