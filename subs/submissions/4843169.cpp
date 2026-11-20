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
#define dbg(...)                                           \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" \
         << ": ";                                          \
    dbgh(__VA_ARGS__)
#else
#define cerr   \
    if (false) \
    cerr
#define dbg(...)
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

constexpr int maxn = 1e6 + 5;

int n;
long sm, siz[maxn], arr[maxn];
pair<long, int> ans {1e18, -1};
vector<int> graph[maxn];

void pdfs(int u) {
    siz[u] = arr[u];
    for (auto& v : graph[u]) {
        graph[v].erase(find(begin(graph[v]), end(graph[v]), u));
        pdfs(v);
        siz[u] += siz[v];
    }
}

void dfs(int u, int p = -1) {
    long cans = 0;
    if (p != -1) {
        cans = siz[p];
    }
    for (auto& v : graph[u]) {
        cans = max(cans, siz[v]);
    }
    ans = min(ans, pair<long, int> {cans, u});
    for (auto& v : graph[u]) {
        siz[u] -= siz[v];
        siz[v] += siz[u];
        dfs(v, u);
        siz[v] -= siz[u];
        siz[u] += siz[v];
    }
}

int LocateCentre(int _n, int _arr[], int us[], int vs[]) {
    n = _n;
    copy(_arr, _arr + n, arr);
    sm = accumulate(arr, arr + n, long(0));
    for (int i = 0; i < n - 1; i++) {
        graph[us[i]].push_back(vs[i]);
        graph[vs[i]].push_back(us[i]);
    }
    pdfs(0);
    dfs(0);
    dbg(ans.first);
    return ans.second;
}