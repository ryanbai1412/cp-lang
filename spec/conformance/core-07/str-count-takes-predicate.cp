// expect: ok, output "2"
// rule: 07.9.1 06.1.6
// A `Str` counts with the one `count(p)` every sequence has; there is no `count(c: Char)` beside it (rebuild 7, H-COUNT-STR, decision 350).
print("abca".count(c => c == 'a'))
