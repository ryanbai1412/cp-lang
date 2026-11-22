# CCC 2002 S4 Bridge Crossing: people cross in order, groups of at most m at a time, a group takes
# as long as its slowest member. Minimize the total and print the groups.
let m = read Int
let people = read [n] (name: Str, t: Int)

# cost[c]: the least time to get everyone from cut c onward across.
@memo(array)
fn cost(c: people.cuts) -> (total: Int, group: Range) =
    if c == people.cuts.last: (total: 0, group: c..c)
    else: min(for g in people[c..].prefixes(max_len: m): (total: g.max(of: .t) + cost(g.end).total, group: g.range))

print("Total Time: {cost(people.cuts.first).total}")
for g in iterate(people.cuts.first, c => cost(c).group.end).until(people.cuts.last).map(c => cost(c).group):
    print(people[g].map(.name))
