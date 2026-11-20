// cp: 0.1 bea787c+dirty
let x = 0.5
print(close(x, 0.5, 1e-9), sign(x, 1e-9), x == 0.0)
print("{sqrt(x):.3f} {sin(x):.3f} {cos(x):.3f} {tan(x):.3f} {atan2(x, 1.0):.3f} {exp(x):.3f} {ln(x):.3f} {hypot(3.0, 4.0):.1f} {PI:.2f} {floor(x)} {ceil(x)} {round(x)} {trunc(x)} {F64(3):.1f}")
