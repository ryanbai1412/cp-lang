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

const long mod = 1e9;
const int maxn = 1e5 + 5;

long ans = 0;
int asiz[maxn], siz[maxn];
vector<int> graph[maxn];

void pdfs(int u, int p = -1) {
    siz[u] = asiz[u];
    for (auto& v : graph[u]) {
        if (v != p) {
            pdfs(v, u);
            siz[u] += siz[v];
        }
    }
}

void dfs(int u, int p = -1) {
    for (auto& v : graph[u]) {
        if (v != p) {
            ans += long(siz[0] - siz[v]) * siz[v];
            dfs(v, u);
        }
    }
}

long solve(int n, int *arrx, int *arry) {
    ans = 0;
    memset(asiz, 0, sizeof(asiz));
    memset(siz, 0, sizeof(siz));
    for (int i = 0; i < n; i++) {
        graph[i].clear();
    }
    map<int, vector<int>> m;
    for (int i = 0; i < n; i++) {
        m[arrx[i]].push_back(arry[i]);
    }
    int cind = 0;
    map<pair<int, int>, int> xtcomp;
    for (auto& [k, v] : m) {
        sort(begin(v), end(v));
        for (int l = 0, r = 0; l < sz(v); l = r) {
            for (; r < sz(v) && v[r] - r == v[l] - l; r++) {
                xtcomp[{k, v[r]}] = cind;
                asiz[cind]++;
                auto it = xtcomp.find({k - 1, v[r]});
                if (it != xtcomp.end()) {
                    int u = it->second;
                    if (!sz(graph[u]) || graph[u].back() != cind) {
                        graph[u].push_back(cind);
                        graph[cind].push_back(u);
                    }
                }
            }
            cind++;
        }
    }
    pdfs(0);
    dfs(0);
    return ans;
}

int DistanceSum(int n, int *arrx, int *arry) {
    return (solve(n, arrx, arry) + solve(n, arry, arrx)) % mod;
}