// expect: ok, output "1\n2"
// rule: 06.5.1
// input: "2\n"
// A case may change a shared `var`; the change carries to the next case (round 25, H09).
var seen = 0
@!multitest
seen += 1
print(seen)
