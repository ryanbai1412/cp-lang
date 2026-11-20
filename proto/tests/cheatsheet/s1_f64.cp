let n: Int = read()
let h: F64 = F64(n) + 0.5
let r = sqrt(3.0)
let tiny: F64 = -0.0001
let same = if close(0.1 + 0.2, 0.3, 1e-9) and sign(tiny, 1e-3) == 0: "yes" else: "no"
print("{h:.6f} {r:.2f} {floor(2.5)} {ceil(2.5)} {round(2.4)} {trunc(2.9)} {tiny:.2f} {PI:.6f} {hypot(3.0, 4.0):.3f} {same}")
