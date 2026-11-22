// expect: ok, output "1 3 5 -1 0 6"
// rule: 07.9.5
// Str.find and Str.rfind support both Char elements and Str substrings.
let s: Str = "banana"
print(s.find("an") ?? -1, s.rfind("an") ?? -1, s.rfind('a') ?? -1, s.find("zz") ?? -1, s.find("") ?? -1, s.rfind("") ?? -1)
