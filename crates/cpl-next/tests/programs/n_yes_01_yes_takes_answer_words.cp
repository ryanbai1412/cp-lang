// expect: ok, output "YES NO Impossible ans=Yes"
// rule: R10.3.3
// `yes(ok, answers:)` is the answer word for `ok`, from `answers:` when given, else `("YES", "NO")`.
let w = yes(false, answers: ("Possible", "Impossible"))
print(yes(true), yes(false), w, "ans={yes(true, answers: ("Yes", "No"))}")
