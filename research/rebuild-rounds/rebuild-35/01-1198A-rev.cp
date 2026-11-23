// CF 1198A MP3, revised: the file also stores a 32-bit header entry per distinct value,
// so K distinct values cost n * ceil(log2 K) + 32 * K bits.
let n: 1..=4e5 = read()
let bytes: 4..=1e8 = read()
var a = read<[n]0..=1e9>()
a.sort()
fn fits(k: Int) -> Bool = n * bit_width(k - 1) + 32 * k <= 8 * bytes
let longest = a.maximal(Counter<Int>(0), ok: c => fits(c.len)).max(by: .len)!
print(n - longest.len)
