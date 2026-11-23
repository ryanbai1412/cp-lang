// expect: ok, output "a bc d"
// rule: 05.7.10
// The file body is a fn: a top-level `var` declared after its source may hold a lent piece there. Only a fn may not store into it.
let s = "a,bc,d"
var words = List<Str>()
var l = 0
while l < s.len:
    let j = s[l..].search(',') ?? s.len
    words.push(s[l..j])
    l = j + 1
print(words)
