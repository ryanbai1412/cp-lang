// expect: ok, output "NO YES\nxy"
// rule: R02.3.1
// `Str.is_empty` is std.list's, so a std module that builds text can ask it (std.map's Show).
let s = "ab"
print(yes(s.is_empty), yes("".is_empty))
var o = ""
if o.is_empty: o += "x"
if not o.is_empty: o += "y"
if o.is_empty: o += "z"
print(o)
