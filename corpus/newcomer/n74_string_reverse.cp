let s: Str = read()
var t: List<Char> = []
for i in 0..s.len:  // expect: push_loop
    t.push(s[s.len - 1 - i])
print(Str(t))
