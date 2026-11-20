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

const int maxn = 1505;

int n, par[maxn], cnt[maxn][maxn];

int find(int u) {
    return par[u] < 0 ? u : (par[u] = find(par[u]));
}

int siz(int u) {
    return -par[find(u)];
}

void initialize(int _n) {
    n = _n;
    memset(par, -1, sizeof(par));
}

int hasEdge(int u, int v) {
    u = find(u);
    v = find(v);
    cnt[u][v]++;
    cnt[v][u]++;
    if (cnt[u][v] == siz(u) * siz(v)) {
        par[v] += par[u];
        par[u] = v;
        for (int i = 0; i < n; i++) {
            cnt[v][i] += cnt[u][i];
        }
        for (int i = 0; i < n; i++) {
            if (i != v) {
                cnt[i][v] += cnt[i][u];
            }
        }
        return 1;
    }
    return 0;
}