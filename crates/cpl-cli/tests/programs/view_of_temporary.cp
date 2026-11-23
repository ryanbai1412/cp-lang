// A view returned by a call keeps the temporary it views alive (C++ ran ASan-clean only after the fix).
fn mk(n: Int) -> List<Int> = [i * 10 for i in 0..n]

print(mk(4).take(2))
let v = mk(5).take(3)
print(v)
print(mk(6)[1..3])
print(chain([1, 3, 2], 4..=6).take(4))
for k in 1..3:
    print(chain([1, 3, 2], 4..=k + 4).take(k + 1))
