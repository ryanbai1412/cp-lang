// `rfind` and `in` take a `Str` or one `Char`, as `find` does.
let s = "banana"
print(s.rfind('n'), s.rfind("an"), s.rfind("x"), s.rfind("banana"), s.rfind(""), s.rfind('z'))
print(s.find('n'), s.find("an"), "".rfind("a"), "".rfind(""))
print("an" in s, "nab" in s, 'n' in s, 'x' in s, "" in s, "banana!" in s)
print("an" not in s, "xy" not in s, 'b' not in s)
let (t, p): (Str, Str) = read()
print(p in t, t.rfind(p), t.find(p))
if p in t and "zz" not in t: print("both")
print(["a" + w for w in ["n", "x"] if "a" + w in s])
