// expect: ok, output "3 3 3 8 64"
// rule: 11.3.3
// `popcount`, `ctz`, `bit_width`, `bit_ceil` are core builtins (rebuild round 17 H05).
print(popcount(13), ctz(8), bit_width(5), bit_ceil(5), popcount(-1))
