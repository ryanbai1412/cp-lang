// expect: ok, output "0 1 2\n0 2\n1\n0 1 2\n1 3"
let s = "aaaa"
print([i for i in s.find_all("aa")])
print([i for i in find_all("abab", "ab")])
for i in s.find_all("a"):
    print(i + 1)
    break
print([i for i in "abc".find_all("")])
print([i for i in [5, 1, 2, 1, 2].find_all([1, 2])])
