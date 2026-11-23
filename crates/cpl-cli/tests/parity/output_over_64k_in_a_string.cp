// Over 64 KiB built inside one "{..}" string after 60 KiB already printed:
// the C++ flushed the string's part first (acceptance run 6).
for i in 0..6000: print("line {i}")
let xs = [i for i in 0..20000]
print("{xs}")
print(xs.len, "items", xs)
print("tail {xs.rev}")
print("end")
