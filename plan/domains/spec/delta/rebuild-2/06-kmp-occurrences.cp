// String matching: text s and pattern p (lowercase, lengths <= 1e6). Print the number
// of occurrences of p in s, then their 1-based starts, via the prefix function of p#s.
let s = read<Str>()
let p = read<Str>()
let t = p + "#" + s
let pi = prefix_function(t)
let starts = [i - 2 * p.len + 1 for i in t.keys if pi[i] == p.len]
print(starts.len)
print(starts)
