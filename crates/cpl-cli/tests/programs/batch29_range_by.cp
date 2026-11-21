let n: Int = read()
fn top() -> Int:
    print("top")
    n
for i in (0..n) by 2: print(i)
for i in (1..=n) by 3: print(i)
for i in (0..n).rev by 2: print(i)
for i in (1..=n).rev by 3: print(i)
for i in (0..top()).rev by 3: print(i)
for i in (n..n).rev by 2: print(i)
for i in (n..0).rev by 2: print(i)
for i in 0..n by 3: print(i)
print([i for i in (0..n) by 2], sum(i for i in (0..n).rev by 3))
let down = (0..6).rev by 2
print(down)
