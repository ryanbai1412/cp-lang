// CF 1198A MP3: clamp to the window of sorted values with the fewest changes.
let n: 1..=4e5 = read()
let bytes: 1..=1e8 = read()
var a = read<[n]0..=1e9>()
a.sort()
let distinct = 1 << min(30, 8 * bytes / n)
let longest = a.maximal(Counter<Int>(0), ok: c => c.len <= distinct).max(by: .len)!
print(n - longest.len)
