// Editorial idea: the shortest path in f(G) stays inside rows w and y, the stars
// a_a - a_b, and the swaps a_b - b_a, so BFS over those 3n + 2m vertices.
#include <bits/stdc++.h>
using namespace std;

int main() {
    cin.tie(nullptr)->sync_with_stdio(false);
    int t;
    cin >> t;
    vector<int> ea, eb, head, nxt, to, dist, q;
    while (t--) {
        int n, m, w, x, y, z;
        cin >> n >> m >> w >> x >> y >> z;
        --w, --x, --y, --z;
        ea.resize(m), eb.resize(m);
        for (int i = 0; i < m; i++) cin >> ea[i] >> eb[i], --ea[i], --eb[i];
        auto id = [&](int p, int c, int e, int side) {
            if (p == w) return c;
            if (p == y) return n + c;
            if (p == c) return 2 * n + p;
            return 3 * n + 2 * e + side;
        };
        int V = 3 * n + 2 * m;
        head.assign(V, -1);
        nxt.clear(), to.clear();
        auto link = [&](int u, int v) {
            to.push_back(v), nxt.push_back(head[u]), head[u] = (int)to.size() - 1;
            to.push_back(u), nxt.push_back(head[v]), head[v] = (int)to.size() - 1;
        };
        for (int e = 0; e < m; e++) {
            int a = ea[e], b = eb[e];
            int ab = id(a, b, e, 0), ba = id(b, a, e, 1);
            link(id(w, a, e, 0), id(w, b, e, 0));
            if (y != w) link(id(y, a, e, 0), id(y, b, e, 0));
            link(id(a, a, e, 0), ab);
            link(id(b, b, e, 0), ba);
            link(ab, ba);
        }
        dist.assign(V, -1);
        q.assign(1, id(w, x, 0, 0));
        dist[q[0]] = 0;
        for (size_t i = 0; i < q.size(); i++) {
            int u = q[i];
            for (int k = head[u]; k != -1; k = nxt[k])
                if (dist[to[k]] < 0) dist[to[k]] = dist[u] + 1, q.push_back(to[k]);
        }
        cout << dist[id(y, z, 0, 0)] << '\n';
    }
}
