// expect: ok, output "3 4"
// rule: 2.7.6
let r = (total: 3, take: 4)
let (total:, take:) = r
print(total, take)
