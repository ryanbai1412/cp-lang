// CSES 1753 String Matching: count occurrences of pattern p in text t.
let t: Str = read()
let p: Str = read()
let z = z_function(p + "#" + t)
print(count(z[i] >= p.len for i in p.len + 1..z.len))
