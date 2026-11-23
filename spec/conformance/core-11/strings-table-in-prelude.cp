// expect: ok, output "3 1 2"
// rule: 11.3.2
// `borders` and `overlap` are in lib/strings.md's §3.2 table, so they are prelude names.
print(borders("abacaba").collect(), overlap("abc", "bcd"))
