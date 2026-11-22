// `rfind` and `in` take a `Str` or one `Char`, as `find` does.
let s = "banana"
print(s.rfind('n') ?? -1, s.rfind("an") ?? -1, s.rfind("x") ?? -1, s.rfind("banana") ?? -1, s.rfind("") ?? -1, s.rfind('z') ?? -1)
print(s.find('n') ?? -1, s.find("an") ?? -1, "".rfind("a") ?? -1, "".rfind("") ?? -1)
print(yes("an" in s), yes("nab" in s), yes('n' in s), yes('x' in s), yes("" in s), yes("banana!" in s))
print(yes("an" not in s), yes("xy" not in s), yes('b' not in s))
let (t, p): (Str, Str) = read()
print(yes(p in t), t.rfind(p) ?? -1, t.find(p) ?? -1)
if p in t and "zz" not in t: print("both")
print(["a" + w for w in ["n", "x"] if "a" + w in s])
