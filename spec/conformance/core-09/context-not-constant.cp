// expect: E0801
// rule: 09.1.2
let s: Str = read()
with io(sep: s): // <- here
    print(1, 2)
