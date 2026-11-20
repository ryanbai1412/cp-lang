"""Deterministic, fully materialized example executions (not compiler output)."""

import copy
import json
from pathlib import Path


def variable(name, kind, type_name, **extra):
    return dict(name=name, kind=kind, type=type_name, **extra)


def base(title, description, filename, source, variables):
    return dict(schemaVersion=1, title=title, description=description,
                source=dict(file=filename, code=source), variables=variables,
                runs={"fast": dict(label="Execution", steps=[])})


def step(steps, line, label, values, *, checkpoint=None, focus=None, locals=None, stage="Compute"):
    steps.append(dict(id=len(steps), line=line, label=label, stage=stage,
                      values=copy.deepcopy(values), locals=locals or {},
                      **({"checkpoint": checkpoint} if checkpoint is not None else {}),
                      **({"focus": focus} if focus else {})))


def knapsack():
    trace = base("0/1 knapsack", "A reused item hides in a single DP transition. Compare the optimized run with its reference.",
                 "knapsack.cp", """fn knapsack(w: [I]Int, v: [I]Int, cap: Int) -> Int:
    var dp: [I.bounds, 0..=cap]Int = 0
    for i in I:
        for c in 1..=cap:
            dp[i.r, c] = dp[i.l, c]
            if c >= w[i]:
                let rest = c - w[i]
                dp[i.r, c] max=
                    dp[i.r, rest] + v[i]
    dp[I.last.r, cap]

// spec uses dp[i.l, rest] on line 9
// Each item may be chosen at most once.
""", {
        "dp": variable("dp", "dp", "[I.bounds, 0..=cap]Int", rowLabels=["∅", "w=2", "w=3", "w=4", "w=5"], colLabels=list(range(9)), rowDomain="I.bounds", colDomain="0..=cap"),
        "weights": variable("weights", "array", "[I]Int", labels=list(range(4))),
        "values": variable("values", "array", "[I]Int", labels=list(range(4))),
        "answer": variable("answer", "scalar", "?Int")})
    trace["runs"]["fast"]["label"] = "knapsack"
    trace["runs"]["slow"] = dict(label="knapsack_spec", steps=[])
    trace["compare"] = ["dp", "answer"]
    for name, run in trace["runs"].items():
        dp = [[0] * 9 for _ in range(5)]
        state = dict(dp=dp, weights=[2, 3, 4, 5], values=[3, 4, 5, 6], answer=None)
        step(run["steps"], 2, "Initialize the DP table", state, checkpoint="init", stage="Input")
        for i in range(1, 5):
            for c in range(1, 9):
                dp[i][c] = dp[i - 1][c]
                if c >= state["weights"][i - 1]:
                    rest = c - state["weights"][i - 1]
                    dp[i][c] = max(dp[i][c], dp[i if name == "fast" else i - 1][rest] + state["values"][i - 1])
                # Spec has extra internal events: alignment is semantic, not by step offset.
                if name == "slow":
                    prior = copy.deepcopy(state)
                    prior["dp"][i][c] = run["steps"][-1]["values"]["dp"][i][c]
                    step(run["steps"], 6, "Check item capacity", prior, locals=dict(i=i, c=c))
                step(run["steps"], 9 if c >= state["weights"][i - 1] else 5,
                     f"Resolve dp[{i}][{c}]", state, checkpoint=f"cell:{i}:{c}",
                     focus=dict(variable="dp", path=[i, c]), locals=dict(i=i, c=c, weight=state["weights"][i - 1]))
        state["answer"] = dp[4][8]
        step(run["steps"], 10, "Return the best value", state, checkpoint="return", stage="Output")
    return trace


def dijkstra():
    trace = base("Shortest paths", "Follow the frontier as Dijkstra settles a weighted graph.", "dijkstra.cp", """fn shortest_paths(g: Graph<V, Int>, start: V) -> [V]?Int:
    var dist: [V]?Int = none
    dist[start] = 0
    var queue = MinHeap<(Int, V)>()
    queue.push((0, start))
    while let (d, u) = queue.pop():
        if dist[u] != d: continue
        for (v, w) in g[u]:
            if dist[v].is_none or d + w < dist[v]!:
                dist[v] = d + w
                queue.push((d + w, v))
    dist
""", {"graph": variable("g", "graph", "Graph<V, Int>", directed=False),
        "dist": variable("dist", "array", "[V]?Int", labels=list("ABCDEFG")),
        "queue": variable("frontier", "array", "MinHeap<(Int, V)>")})
    nodes = [dict(id=i, label=label, x=x, y=y) for i, (label, x, y) in enumerate(zip("ABCDEFG", [60, 195, 195, 335, 335, 465, 465], [160, 65, 255, 65, 255, 125, 280]))]
    links = [(0, 1, 4), (0, 2, 2), (1, 2, 1), (1, 3, 5), (2, 4, 3), (3, 4, 2), (3, 5, 3), (4, 5, 6), (4, 6, 4), (5, 6, 1)]
    graph = dict(nodes=nodes, edges=[dict(id=i, **{"from": a, "to": b}, weight=w) for i, (a, b, w) in enumerate(links)])
    state = dict(graph=graph, dist=[0] + [None] * 6, queue=[[0, "A"]])
    steps = trace["runs"]["fast"]["steps"]
    step(steps, 3, "Start at vertex A", state, stage="Input")
    queue, settled = [(0, 0)], set()
    import heapq
    while queue:
        d, u = heapq.heappop(queue)
        if u in settled:
            continue
        settled.add(u)
        nodes[u]["state"] = "visited"
        state["queue"] = [[a, chr(65+b)] for a, b in sorted(queue)]
        step(steps, 5, f"Settle vertex {chr(65+u)} at distance {d}", state, focus=dict(variable="graph", path=["nodes", u]), locals=dict(u=chr(65+u), d=d))
        for edge in graph["edges"]:
            a, b = edge["from"], edge["to"]
            v = b if a == u else a if b == u else None
            if v is None or v in settled:
                continue
            candidate = d + edge["weight"]
            if state["dist"][v] is None or candidate < state["dist"][v]:
                state["dist"][v] = candidate
                heapq.heappush(queue, (candidate, v))
                nodes[v]["state"] = "frontier"
                state["queue"] = [[a, chr(65+b)] for a, b in sorted(queue)]
                step(steps, 10, f"Relax {chr(65+u)} → {chr(65+v)}", state, focus=dict(variable="graph", path=["edges", edge["id"]]), locals=dict(u=chr(65+u), v=chr(65+v), candidate=candidate))
    step(steps, 12, "All shortest paths are settled", state, stage="Output")
    return trace


def tree():
    trace = base("Subtree sizes", "A postorder walk turns a rooted tree into subtree counts.", "subtree.cp", """fn subtree_sizes(tree: Rooted<V>) -> [V]Int:
    var size: [V]Int = 1
    fn visit(u: V):
        for v in tree.children(u):
            visit(v)
            size[u] += size[v]
    visit(tree.root)
    size
""", {"tree": variable("tree", "tree", "Rooted<V>", root=0), "size": variable("size", "array", "[V]Int", labels=list("ABCDEFG")), "stack": variable("call stack", "array", "List<V>")})
    nodes = [dict(id=i, label=c) for i, c in enumerate("ABCDEFG")]
    edges = [dict(id=i, **{"from": a, "to": b}) for i, (a, b) in enumerate([(0, 1), (0, 2), (1, 3), (1, 4), (2, 5), (2, 6)])]
    state = dict(tree=dict(nodes=nodes, edges=edges), size=[1]*7, stack=[])
    steps = trace["runs"]["fast"]["steps"]
    step(steps, 2, "Initialize one vertex per subtree", state, stage="Input")
    def visit(u):
        state["stack"].append(chr(65+u))
        nodes[u]["state"] = "frontier"
        step(steps, 3, f"Enter subtree {chr(65+u)}", state, focus=dict(variable="tree", path=["nodes", u]), locals=dict(u=chr(65+u)))
        for e in edges:
            if e["from"] == u:
                v = e["to"]
                visit(v)
                state["size"][u] += state["size"][v]
                step(steps, 6, f"Accumulate size[{chr(65+u)}] += {state['size'][v]}", state, focus=dict(variable="size", path=[u]), locals=dict(u=chr(65+u), v=chr(65+v)))
        nodes[u]["state"] = "visited"
        state["stack"].pop()
        step(steps, 5, f"Finish subtree {chr(65+u)}", state, focus=dict(variable="tree", path=["nodes", u]))
    visit(0)
    step(steps, 8, "Return subtree sizes", state, stage="Output")
    return trace


def grid():
    trace = base("Maze breadth-first search", "Watch a wavefront find the shortest route around blocked cells.", "maze.cp", """fn shortest_path(maze: [Row, Col]Char, start: Cell, goal: Cell) -> ?Int:
    var distance: [Row, Col]?Int = none
    var queue: Deque<Cell> = [start]
    distance[start] = 0
    while let cell = queue.pop_front():
        for next in cell.adj4:
            if maze[next] == '#' or distance[next].is_some: continue
            distance[next] = distance[cell]! + 1
            queue.push(next)
    distance[goal]
""", {"maze": variable("maze", "grid", "[Row, Col]Char", rowDomain="Row", colDomain="Col"), "distance": variable("distance", "dp", "[Row, Col]?Int"), "queue": variable("frontier", "array", "Deque<Cell>")})
    maze = [list(s) for s in ["S..#....", ".#.#.##.", ".#......", "...##.#.", ".#....#.", "...#...G"]]
    dist = [[None]*8 for _ in range(6)]
    dist[0][0] = 0
    queue = [(0, 0)]
    state = dict(maze=maze, distance=dist, queue=queue)
    steps = trace["runs"]["fast"]["steps"]
    step(steps, 4, "Begin at the entrance", state, stage="Input")
    while queue:
        r, c = queue.pop(0)
        for y, x in [(r-1,c),(r+1,c),(r,c-1),(r,c+1)]:
            if 0 <= y < 6 and 0 <= x < 8 and maze[y][x] != "#" and dist[y][x] is None:
                dist[y][x] = dist[r][c] + 1
                queue.append((y, x))
                step(steps, 8, f"Reach row {y}, column {x}", state, focus=dict(variable="distance", path=[y,x]), locals=dict(row=y, col=x, distance=dist[y][x]))
    step(steps, 10, "Shortest path: 12 moves", state, stage="Output")
    return trace


def array():
    trace = base("Insertion sort", "An in-place array, a moving boundary, and every write made visible.", "sort.cp", """fn insertion_sort(mut a: [0..n]Int):
    for i in 1..n:
        let key = a[i]
        var j = i
        while j > 0 and a[j - 1] > key:
            a[j] = a[j - 1]
            j -= 1
        a[j] = key
""", {"a": variable("a", "array", "[0..n]Int", labels=list(range(8))), "key": variable("key", "scalar", "?Int")})
    state = dict(a=[7, 3, 8, 2, 6, 1, 5, 4], key=None)
    steps = trace["runs"]["fast"]["steps"]
    step(steps, 1, "Read eight unsorted values", state, stage="Input")
    a = state["a"]
    for i in range(1, len(a)):
        key, j = a[i], i
        state["key"] = key
        step(steps, 3, f"Pick key {key}", state, focus=dict(variable="a", path=[i]), locals=dict(i=i,j=j))
        while j > 0 and a[j-1] > key:
            a[j] = a[j-1]
            step(steps, 6, f"Shift {a[j]} one place right", state, focus=dict(variable="a", path=[j]), locals=dict(i=i,j=j))
            j -= 1
        a[j] = key
        step(steps, 8, f"Insert {key} at position {j}", state, focus=dict(variable="a", path=[j]), locals=dict(i=i,j=j))
    step(steps, 8, "Array is sorted in place", state, stage="Output")
    return trace


def all_samples():
    return [knapsack(), dijkstra(), tree(), grid(), array()]


if __name__ == "__main__":
    directory = Path(__file__).parent / "samples"
    directory.mkdir(exist_ok=True)
    samples = all_samples()
    for name, trace in zip(["knapsack", "dijkstra", "tree", "grid", "array"], samples):
        (directory / f"{name}.json").write_text(json.dumps(trace, ensure_ascii=False, indent=2) + "\n")
