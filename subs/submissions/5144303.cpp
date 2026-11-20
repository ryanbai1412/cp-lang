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

using ld = long double;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

constexpr int maxn = 65;

int n, m, par[maxn], cpar[maxn];
ld ans, kv, arr[maxn], kpow[maxn];
vector<int> graph[maxn];

ld dfs(int u) {
    ld cans = arr[u];
    for (auto& v : graph[u]) {
        cans += kv * dfs(v);
    }
    return cans;
}

ld csolve() {
    for (int i = 0; i < n; i++) {
        graph[i].clear();
        dbg(cpar[i]);
    }
    vector<int> cyc;
    {
        bool cvis[n] {};
        int u = 0;
        do {
            cvis[u] = true;
            cyc.push_back(u);
            u = cpar[u];
        } while (!cvis[u]);
        cyc = vector<int>(find(begin(cyc), end(cyc), u), cyc.end());
        if (find(begin(cyc), end(cyc), 0) == cyc.end()) {
            for (int i = 0; i < n; i++) {
                graph[cpar[i]].push_back(i);
            }
            return dfs(0);
        }
    }
    bool icyc[n] {};
    for (auto& a : cyc) {
        icyc[a] = true;
    }
    for (int i = 0; i < n; i++) {
        if (!icyc[i]) {
            graph[cpar[i]].push_back(i);
        }
    }
    rotate(begin(cyc), find(begin(cyc), end(cyc), 0), end(cyc));
    reverse(cyc.begin() + 1, cyc.end());
    ld cans = 0;
    for (int i = 0; i < sz(cyc); i++) {
        cans += dfs(cyc[i]) * kpow[i] / (1 - kpow[sz(cyc)]);
    }
    return cans;
}

void rdfs(int ind, int cm) {
    if (ind == n) {
        dbg("A");
        ans = max(ans, csolve());
        return;
    }
    cpar[ind] = par[ind];
    rdfs(ind + 1, cm);
    if (ind && cm < m) {
        for (int i = 0; i < n; i++) {
            cpar[ind] = i;
            rdfs(ind + 1, cm + 1);
        }
    }
}

void solve() {
    cin >> n >> m >> kv;
    for (int i = 0; i < n; i++) {
        cin >> par[i];
        par[i]--;
    }
    for (int i = 0; i < n; i++) {
        cin >> arr[i];
    }
    kpow[0] = 1;
    for (int i = 0; i + 1 < maxn; i++) {
        kpow[i + 1] = kpow[i] * kv;
    }
    memcpy(cpar, par, sizeof(par));
    cpar[1] = 0;
    rdfs(0, 0);
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    cout << fixed << setprecision(2);
    solve();
}