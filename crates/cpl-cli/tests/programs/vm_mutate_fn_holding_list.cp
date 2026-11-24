// Growing, sorting and clearing a list whose elements hold a fn: the vm
// mutates it in place, as no key of a set or map is compared.
let n: Int = read()
struct Job:
    rank: Int
    run: (Int) -> Int
var jobs: List<Job> = []
jobs.reserve(n)
for k in 1..=n: jobs.push(Job(n - k, x => x + k))
jobs.sort(by: job => job.rank)
print(jobs.map(job => job.run(100)))
jobs.clear()
print(jobs.len)
