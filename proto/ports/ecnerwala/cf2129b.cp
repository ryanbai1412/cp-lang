@!multitest
domain I = read(1..=5000)
let p: [I]Int = read()
print(
    sum(
        min(
            count(p[j] > p[i] for j in I.before(i)),
            count(p[j] > p[i] for j in I.after(i)),
        ) for i in I
    )
)
