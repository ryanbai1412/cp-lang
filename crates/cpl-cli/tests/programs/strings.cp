// `Str` is bytes and `Char` is one byte: indexing is O(1) and there is
// no UTF-8 anywhere.
let s: Str = read()
let t: Str = read()
print(s, s.len, s[0], s[s.len - 1])
print(s + t, s + "-" + t, yes(s == t), yes(s < t), yes(s != t))
var up: Str = ""
for c in s:
    if 'a' <= c <= 'z':
        up.push('A' + (c - 'a'))
    else:
        up.push(c)
print(up)
var count: [26]Int = 0
for c in s:
    if 'a' <= c <= 'z':
        count[c - 'a'] += 1
print(count)
print('a' + 2, 'z' - 'a', yes('a' < 'b'), "tab\there", "quote\"d", "back\\slash")
print("{s} has {s.len} bytes; first is {s[0]}")
print("braces {{like}} these", "what??!")
var rev: Str = ""
for i in (0..s.len).rev:
    rev.push(s[i])
print(rev, yes(rev == s), yes(s.contains('l')), yes(s.contains('q')))
let words: [3]Str = read()
print(words)
var sorted_words: List<Str> = []
for w in words:
    sorted_words.push(w)
sorted_words.sort()
print(sorted_words)
// Bytes over 127 come in and go out untouched.
let raw: Str = read()
print(raw, raw.len)
