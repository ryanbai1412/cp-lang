fn cost(j: Int, i: Int) -> I128 = I128(j * i)
let nodes = [0, 1, 2, 3]
print(aliens(2, solve: pen => monge_path(nodes, cost: (j, i) => cost(j, i) + pen)))
print(aliens(2, solve: (pen: I128) => monge_path(nodes, cost: (j, i) => cost(j, i) + pen)))
