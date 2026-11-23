// `Bool` is not a token type, whether `read` is told so or infers it
// from the binding (R10.2.3).
let a = read<Bool>()
let b: Bool = read()
let c: [3]Bool = read()
print(yes(a and b and c[0]))
