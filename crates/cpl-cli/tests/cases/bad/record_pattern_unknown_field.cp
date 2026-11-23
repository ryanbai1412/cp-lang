// error: the record has no field `tot`
let r = (total: 3, take: 4)
let (tot: u, ..) = r
print(u)
