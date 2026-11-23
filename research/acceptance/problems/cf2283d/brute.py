import sys
def cases(text):
    it = iter(text.split()); t = int(next(it))
    for _ in range(t):
        n = int(next(it))
        yield n, [(int(next(it)), int(next(it))) for _ in range(n - 1)]
def path(n, edges, a, b):
    adj = [[] for _ in range(n + 1)]
    for u, v in edges: adj[u].append(v); adj[v].append(u)
    par = [0] * (n + 1); par[a] = a; stack = [a]
    while stack:
        u = stack.pop()
        for w in adj[u]:
            if not par[w]: par[w] = u; stack.append(w)
    out = [b]
    while out[-1] != a: out.append(par[out[-1]])
    return out
def score(n, edges, k):
    color = ["white"] * (n + 1)
    for i in range(1, n + 1):
        j = k - i
        if 1 <= j <= n and j != i:
            color[i] = color[j] = "black"
            for v in path(n, edges, i, j):
                if color[v] == "white": color[v] = "gray"
    return color.count("black"), color.count("gray")
def best(n, edges):
    return max(range(1, 2 * n + 1), key=lambda k: score(n, edges, k))
if __name__ == "__main__":
    for n, edges in cases(sys.stdin.read()): print(best(n, edges))
