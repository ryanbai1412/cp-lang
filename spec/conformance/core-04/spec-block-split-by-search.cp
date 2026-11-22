// expect: ok, output "3"
// rule: 04b.1.11
let s = "ab,c,de"
var words = List<Str>()
var l = 0
while l < s.len:
    let j = s[l..].search(',') ?? s.len    // a key of s
    words.push(s[l..j])
    l = j + 1
print(words.len)
