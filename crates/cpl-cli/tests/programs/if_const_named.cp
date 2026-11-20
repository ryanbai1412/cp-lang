// `if const` on a named `const`, and `**=` on a `Mod`.
const FLAG = true
const DEPTH = 3
if const FLAG:
    print("on")
else:
    print("off")
if const DEPTH > 2 and not FLAG: print("deep") else: print("shallow")
type Z = Mod<998244353>
var z: Z = 3
z **= 5
print(z)
