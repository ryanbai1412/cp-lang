// expect: ok, output "8 1024"
// rule: 08.1.5
// An `I128`'s `**` takes an `Int` exponent, as `Pow`'s default right side says, and the result stays an `I128`.
let e = 3
let w = I128(2) ** e
print(w, w ** 3 + I128(512))
