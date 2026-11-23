// `xs!` on `[D]?T` traps at the first `None` cell and names its key
// (R02.4.8), the same in both engines: here `depth[3] is None`.
let n = read<1..=5>()
domain V = 1..=n
var depth: [V]?Int = None
for v in V: if Int(v) != 3: depth[v] = Int(v) * 2
print("filled")
let depth = depth!
print(depth[V(1)])
