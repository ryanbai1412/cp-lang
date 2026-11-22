// cp: 0.1 bea787c+dirty
let x: Int = read()
fn neg(): print("neg")
fn zero(): print("zero")
fn pos(): print("pos")
if x < 0: neg()
elif x == 0: zero()
else: pos()
let sign = if x >= 0: 1 else: -1
print(sign)
