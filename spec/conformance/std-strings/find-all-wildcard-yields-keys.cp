// expect: ok, output "2 4 0 1"
// `find_all_wildcard` yields `self`'s keys, as `find_all` does, and an empty pattern matches at each key, never at the end (R04b.5.2).
domain D = 2..=4
let s: [D]Char = ['a', '?', 'a']
print(s.find_all_wildcard("a").collect(), "ab".find_all_wildcard("").collect())
