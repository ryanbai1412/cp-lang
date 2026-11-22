// Enums order by variant in declaration order, then by payload (decision 28.2).
enum Dir: N, E, S, W

enum Event:
    Open(x: Int)
    Query(x: Int, id: Int)
    Close(x: Int)

print(Dir.N < Dir.S, Dir.W < Dir.E, max(Dir.E, Dir.S))
var es = [Event.Close(1), Event.Query(5, 0), Event.Open(3), Event.Open(1), Event.Query(2, 7)]
es.sort()
for e in es:
    print(e)
var ds = [Dir.W, Dir.N, Dir.S]
ds.sort()
print(ds)
fn key(e: Event) -> (Int, Int):
    match e:
        Event.Open(x) => (x, 0)
        Event.Query(x, _) => (x, 1)
        Event.Close(x) => (x, 2)
print(sorted(es, by: key)[0])
