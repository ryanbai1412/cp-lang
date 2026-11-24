// expect: E0203
// has: "`trim`"
// `@aka(strip)` on std's `trim`, `@aka(rstrip, ..)` on `trim_end`.
let s = " a "
print(s.strip()) // <- here
