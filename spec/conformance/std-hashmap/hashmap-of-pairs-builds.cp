// expect: ok, output "2 4"
// `HashMap(pairs)` builds from `(key, value)` pairs, as Python's `dict(pairs)` and Rust's `HashMap::from` do.
let m = HashMap([(1, 2), (3, 4)])
print(m.len, m[3])
