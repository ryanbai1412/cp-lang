let a = [1, 3, 5, 7]
let lo = (0..a.len).first_true(mid => a[mid] >= 5) ?? a.len
print(lo)
