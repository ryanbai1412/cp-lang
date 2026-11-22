// expect: ok, output "3 YES NO 0 1 2 0\n0 4 0 NO\n3 1 YES\n0 1 2\n0\n1 2"
domain V = 0..5
let nx: [V]V = [V(1), V(2), V(0), V(0), V(3)]
let f = Functional<V>.new(nx)
print(f.cycle_len(V(4)), f.on_cycle(V(1)), f.on_cycle(V(3)), f.root_of(V(4)), f.root_of(V(1)), f.depth(V(4)), f.depth(V(0)))
print(f.after(V(4), 5), f.after(V(4), 0), f.jump_while(V(4), u => f.on_cycle(u))!, f.jump_while(V(0), u => u == V(4)) != None)
print(f.dist(V(4), V(1))!, f.dist(V(2), V(0))!, f.dist(V(1), V(4)) == None)
for c in f.cycles(): print(c)
domain U = 0..3
let ny: [U]U = [U(0), U(2), U(1)]
for c in Functional<U>.new(ny).cycles(): print(c)
