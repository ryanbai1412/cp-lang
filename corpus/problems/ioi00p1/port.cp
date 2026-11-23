// IOI 2000 Palindrome: the fewest characters to insert into a string to make
// it a palindrome. need[j] is the answer for s[i..=j]; rows for i and i + 1
// are all that is ever needed.
let n: 3..=5000 = read()
domain I = 0..n
fn word() -> [I]Char: [I] _ => 'a' + rand(0, 2)
let s: [I]Char = read(gen: word)

var inner: [I]Int = 0  // the row for i + 1
for i in I.rev:
    var need: [I]Int = 0
    for j in I:
        if j > i:
            need[j] = if s[i] == s[j]: inner[j.prev!] else: 1 + min(inner[j], need[j.prev!])
    inner = need
print(inner[I.last])
