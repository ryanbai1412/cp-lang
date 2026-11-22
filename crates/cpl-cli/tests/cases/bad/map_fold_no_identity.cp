// error: `longer` has no identity to start it from
fn longer(x: Int, y: Int) -> Int: if y > x: y else: x
var m: Map<Int, Int> = []
m[4] longer= 2
print(m[4])
