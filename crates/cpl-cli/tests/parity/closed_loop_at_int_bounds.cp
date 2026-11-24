let h = Int.MAX
let m = Int.MIN
let up: Int = read()
let down: Int = read()
for i in h - up..=h:
    if i == h - 1: continue
    print(h - i)
for i in (m..=m + down).rev:
    if i == m + 1: continue
    print(i - m)
for i in m..=h:
    if i >= m + 2: break
    print(i - m)
for i in (m..=h).rev:
    if i <= h - 2: break
    print(h - i)
for i in up..=down: print("none")
for i in (up..=down).rev: print("none")
for i in h..=h: print("once")
for i in (m..=m).rev: print("once")
