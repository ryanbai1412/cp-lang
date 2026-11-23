// expect: E0503 at 4
// rule: 
// has: "elif"
// `else if` is spelled `elif`, and the error writes it (rebuild round 32 FL3).
let a = 1
if a == 0: print(0)
else if a == 1: print(1)
