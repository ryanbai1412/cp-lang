// Tests the declared bounds leave open.
let n: 0..=10 = read()
let k: 0..=10 = read()
if n >= 1 and k < n:
    print(n - k)
