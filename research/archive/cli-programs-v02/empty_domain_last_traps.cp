// The ends of an empty domain stop the program: status 3, not position 0.
let n: Int = read()
domain D = 0..n
print(D.last.to_idx)
