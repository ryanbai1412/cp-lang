#include "bits/extc++.h"

using namespace std;

template <typename T>
void dbgh(const T& t) {
    cerr << t << endl;
}

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t << " | ";
    dbgh(u...);
}

#ifdef DEBUG
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

const int maxn = 1e6 + 5;

struct DSU {
    int cyc, cu, p[maxn];

    DSU() : cyc(false) {
        memset(p, -1, sizeof(p));
    }

    int find(int u) {
        return p[u] < 0 ? u : (p[u] = find(p[u]));
    }

    void merge(int u, int v) {
        u = find(u);
        v = find(v);
        if (u == v) {
            cu = u;
            cyc++;
            return;
        }
        if (p[u] < p[v]) {
            swap(u, v);
        }
        p[v] += p[u];
        p[u] = v;
    }
} dsu, dsus[4];

int n, zans = -1, dcnt[5], big[maxn], imp[4];
vector<int> graph[maxn];
vector<pair<int, int>> edges;

void Init(int _n) {
    n = _n;
    imp[0] = -1;
}

void upd_dcnt(int d) {
    dcnt[min(d, 4)]--;
    dcnt[min(d + 1, 4)]++;
}

void Link(int u, int v) {
    dsu.merge(u, v);
    edges.emplace_back(u, v);
    upd_dcnt(sz(graph[u]));
    upd_dcnt(sz(graph[v]));
    if (sz(graph[u]) >= 3) {
        big[v]++;
    }
    if (sz(graph[v]) >= 3) {
        big[u]++;
    }
    graph[u].push_back(v);
    graph[v].push_back(u);
    if (sz(graph[u]) == 3) {
        big[u]++;
        for (auto& a : graph[u]) {
            big[a]++;
        }
    }
    if (sz(graph[v]) == 3) {
        big[v]++;
        for (auto& a : graph[v]) {
            big[a]++;
        }
    }
    if (imp[0] == -1) {
        int cx = -1;
        if (sz(graph[u]) >= 3) {
            cx = u;
        } else if (sz(graph[v]) >= 3) {
            cx = v;
        }
        if (cx != -1) {
            assert(sz(graph[cx]) == 3);
            for (int i = 0; i < 4; i++) {
                int cy;
                if (i < 3) {
                    cy = graph[cx][i];
                } else {
                    cy = cx;
                }
                imp[i] = cy;
                for (auto& [u, v] : edges) {
                    if (u != cy && v != cy) {
                        dsus[i].merge(u, v);
                    }
                }
            }
        }
    } else {
        for (int i = 0; i < 4; i++) {
            if (u != imp[i] && v != imp[i]) {
                dsus[i].merge(u, v);
            }
        }
    }
}

int CountCritical() {
    if (dcnt[4] >= 2) {
        return 0;
    } else if (imp[0] != -1) {
        int ans = 0;
        for (int i = 0; i < 4; i++) {
            bool cur = !dsus[i].cyc;
            if (dcnt[4] == 1) {
                cur &= sz(graph[imp[i]]) >= 4;
            }
            cur &= big[imp[i]] == dcnt[3] + dcnt[4];
            ans += cur;
        }
        return ans;
    } else {
        if (dsu.cyc > 1) {
            return 0;
        } else if (dsu.cyc == 1) {
            if (zans != -1) {
                return zans;
            }
            zans = 0;
            int u = dsu.cu, p = -1;
            dbg(u);
            do {
                assert(sz(graph[u]) == 2);
                if (p != graph[u][0]) {
                    p = u;
                    u = graph[u][0];
                } else {
                    p = u;
                    u = graph[u][1];
                }
                dbg(u);
                zans++;
            } while (u != dsu.cu);
            return zans;
        }
        assert(!dcnt[4] && !dcnt[3]);
        return n;
    }
}
// The original is the three functions the IOI grader calls. This `main` stands in for the
// grader so the port can be stressed against it: n and the number of calls, then one call per
// line, "u v" for Link(u, v) and "-1 -1" for CountCritical().
int main() {
    int n, calls;
    if (scanf("%d %d", &n, &calls) != 2) return 1;
    Init(n);
    for (int i = 0; i < calls; i++) {
        int u, v;
        if (scanf("%d %d", &u, &v) != 2) return 1;
        if (u == -1) printf("%d\n", CountCritical());
        else Link(u, v);
    }
}
