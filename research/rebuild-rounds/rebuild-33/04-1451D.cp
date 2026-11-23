// cf1451d Circle Game: Utkarsh mirrors along the diagonal, so only the last diagonal point
// (z k, z k) matters: Ashish wins iff (z k, (z + 1) k) is still inside the circle.
@!multitest
let d: 1..=1e5 = read()
let k: 1..=1e5 = read()
var z = floor(F64(d) / (F64(k) * sqrt(2.0)))
while 2 * ((z + 1) * k) ** 2 <= d * d: z += 1
while 2 * (z * k) ** 2 > d * d: z -= 1
print(if (z * k) ** 2 + ((z + 1) * k) ** 2 <= d * d: "Ashish" else: "Utkarsh")
