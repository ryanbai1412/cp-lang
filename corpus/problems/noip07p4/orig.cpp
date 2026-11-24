// Reference for NOIP 2007 P4 (no accepted C++ on DMOJ: its submissions are TLE, WA or RTE;
// written for the corpus). Take one diameter u..v (two farthest-node passes). Every core lies
// on a diameter and all diameters share the center, so the answer is the larger of the deepest
// branch hanging off this diameter and the best window of length <= s on it, where a window
// [i, j] costs max(d(u, i), d(j, v)). Iterative passes, O(n).
#include <bits/stdc++.h>
using namespace std;

static char buf[1 << 25];
int bl = 0, bp = 0;
int gc() {
    if (bp == bl) { bl = fread(buf, 1, sizeof buf, stdin); bp = 0; if (bl <= 0) return -1; }
    return buf[bp++];
}
long long rd() {
    int c = gc();
    while (c != '-' && (c < '0' || c > '9')) c = gc();
    long long x = 0;
    for (; c >= '0' && c <= '9'; c = gc()) x = x * 10 + (c - '0');
    return x;
}

int n;
vector<int> head, nxt, to, wt, par;
vector<long long> dist;

int farthest(int s, const vector<char> &blocked) {
    // BFS order works on a tree: distances are path sums.
    vector<int> q{s};
    dist[s] = 0, par[s] = 0;
    int best = s;
    for (size_t k = 0; k < q.size(); k++) {
        int u = q[k];
        if (dist[u] > dist[best]) best = u;
        for (int e = head[u]; e; e = nxt[e]) {
            int v = to[e];
            if (v == par[u] || blocked[v]) continue;
            par[v] = u, dist[v] = dist[u] + wt[e];
            q.push_back(v);
        }
    }
    return best;
}

int main() {
    n = rd();
    long long s = rd();
    head.assign(n + 1, 0), nxt.assign(2 * n, 0), to.assign(2 * n, 0), wt.assign(2 * n, 0);
    par.assign(n + 1, 0), dist.assign(n + 1, 0);
    int ec = 0;
    for (int i = 1; i < n; i++) {
        int a = rd(), b = rd(), w = rd();
        to[++ec] = b, wt[ec] = w, nxt[ec] = head[a], head[a] = ec;
        to[++ec] = a, wt[ec] = w, nxt[ec] = head[b], head[b] = ec;
    }
    vector<char> none(n + 1, 0);
    int u = farthest(1, none);
    int v = farthest(u, none);
    vector<int> path;
    vector<long long> pd;  // d(u, path[k])
    for (int x = v; x; x = par[x]) path.push_back(x), pd.push_back(dist[x]);
    reverse(path.begin(), path.end()), reverse(pd.begin(), pd.end());
    long long total = pd.back();
    vector<char> on(n + 1, 0);
    for (int x : path) on[x] = 1;
    // Deepest branch off the diameter: a multi-source pass from every path node.
    long long branch = 0;
    {
        vector<int> q;
        vector<long long> d(n + 1, -1);
        for (int x : path) d[x] = 0, q.push_back(x);
        for (size_t k = 0; k < q.size(); k++) {
            int a = q[k];
            branch = max(branch, d[a]);
            for (int e = head[a]; e; e = nxt[e])
                if (d[to[e]] < 0) d[to[e]] = d[a] + wt[e], q.push_back(to[e]);
        }
    }
    long long best = LLONG_MAX;
    for (size_t i = 0, j = 0; i < path.size(); i++) {
        if (j < i) j = i;
        while (j + 1 < path.size() && pd[j + 1] - pd[i] <= s) j++;
        best = min(best, max(pd[i], total - pd[j]));
    }
    printf("%lld\n", max(best, branch));
}
