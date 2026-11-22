// CSES 1753 String Matching: count occurrences of pattern p in text t.
let t = read<Str>()
let p = read<Str>()
print(t.find_all(p).count(i => true))
