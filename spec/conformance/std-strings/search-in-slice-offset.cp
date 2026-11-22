// expect: ok, output "3 5"
let s = "banana"
print(s[2..].search("a") ?? -1, s.rsearch("a") ?? -1)
