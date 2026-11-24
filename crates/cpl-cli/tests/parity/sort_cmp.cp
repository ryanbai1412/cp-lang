struct Job(a: Int, b: Int, id: Int)

fn by_ratio(x: Job, y: Job) -> Bool = x.a * y.b < y.a * x.b

var jobs = [Job(3, 1, 0), Job(1, 2, 1), Job(2, 4, 2), Job(1, 1, 3)]
jobs.sort(cmp: (x, y) => x.a * y.b < y.a * x.b)
print([j.id for j in jobs])

let desc_jobs = jobs.sorted(cmp: (x, y) => by_ratio(y, x))
print([j.id for j in desc_jobs])

let arr: [0..4]Int = [12, 3, 25, 4]
print(arr.sorted(cmp: (a, b) => a % 10 < b % 10))
print(arr[1..4].sorted(cmp: (a, b) => a > b))
