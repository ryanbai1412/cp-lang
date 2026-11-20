// Synthesis #2 rule 5 and the format specifiers: `{v:W}`, `{v:0W}`, `{x:.D}`, `{x:.Df}`, `{n:x}`, `{n:b}`.
let x = 1.0 / 3.0
let w = 42
let neg = -3
let tiny = -0.0001
let big = 1 << 40
print("[{w:5}] [{x:.5}] [{x:.5f}] [{w:05}] [{255:x}] [{5:b}] [{neg:04}] [{neg:5}]")
print("[{w:08b}] [{w:6x}] [{neg:x}] [{neg:06b}] [{big:x}] [{0:b}] [{0:x}]")
print("[{tiny:.3}] [{tiny:08.3f}] [{x:9.4f}] [{w:.2f}] [{-2.5:07.2f}] [{2.5:.0f}] [{x:.1}]")
let hi = 7
let lo = 123
print("{hi}{lo:018}")
print("{w:04}", "{w:1}", "{1000000:3}")
// the shifts are 64-bit on both paths
print(1 << 32, 1 << 62, (1 << 33) >> 1, "{1 << 32:x}", "{(1 << 35) + 1:b}")
