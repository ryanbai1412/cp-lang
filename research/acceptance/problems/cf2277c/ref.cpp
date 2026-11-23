#include <bits/stdc++.h>
using namespace std;
static char buf[1 << 25];
int len, pos;
int rd() {
    while (buf[pos] < '0') pos++;
    int x = 0;
    while (buf[pos] >= '0') x = x * 10 + buf[pos++] - '0';
    return x;
}
int main() {
    len = fread(buf, 1, sizeof(buf) - 1, stdin);
    buf[len] = 0;
    int t = rd();
    vector<int> head, nxt, to, dist, q;
    string out;
    while (t--) {
        int n = rd(), m = rd();
        head.assign(n + 1, -1); nxt.resize(2 * n); to.resize(2 * n); dist.assign(n + 1, -1); q.resize(n);
        for (int i = 0; i < n - 1; i++) {
            int x = rd(), y = rd();
            to[2 * i] = y; nxt[2 * i] = head[x]; head[x] = 2 * i;
            to[2 * i + 1] = x; nxt[2 * i + 1] = head[y]; head[y] = 2 * i + 1;
        }
        // BFS from n without passing through vertex 1: the final 1-n path is 1, x, then an original x-n path avoiding 1
        int qh = 0, qt = 0;
        dist[n] = 0; q[qt++] = n;
        while (qh < qt) {
            int u = q[qh++];
            if (u == 1) continue;
            for (int e = head[u]; e != -1; e = nxt[e])
                if (dist[to[e]] < 0) dist[to[e]] = dist[u] + 1, q[qt++] = to[e];
        }
        int ans = dist[1];
        for (int i = 0; i < m; i++) {
            int v = rd();
            if (dist[v] >= 0) ans = max(ans, dist[v] + 1);
        }
        out += to_string(ans); out += '\n';
    }
    fwrite(out.data(), 1, out.size(), stdout);
}
