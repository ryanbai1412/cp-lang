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

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

constexpr int maxn = 19;

using A = array<long, 1 << maxn>;

int siz[maxn];
vector<int> ord, graph[maxn], subs[maxn + 1][maxn + 1];

void pcomp() {
    subs[0][0] = {0};
    for (int n = 1; n <= maxn; n++) {
        for (int i = 0; i <= n; i++) {
            int perm[n] {};
            fill(perm, perm + i, 1);
            sort(perm, perm + n);
            do {
                int cans = 0;
                for (int j = 0; j < n; j++) {
                    cans |= perm[j] << j;
                }
                subs[n][i].push_back(cans);
            } while (next_permutation(perm, perm + n));
        }
    }
}

void dfs(int u) {
    siz[u] = 1;
    ord.push_back(u);
    for (auto& v : graph[u]) {
        graph[v].erase(find(begin(graph[v]), end(graph[v]), u));
        dfs(v);
        siz[u] += siz[v];
    }
}

A merge(int an, A a, int bn, A b) {
    A ans {};
    for (auto& amask : subs[maxn][an]) {
        vector<int> binds;
        for (int i = 0; i < maxn; i++) {
            if (!on(amask, i)) {
                binds.push_back(i);
            }
        }
        for (auto& bmask : subs[maxn - an][bn]) {
            int brep = 0;
            for (int i = 0; i < sz(binds); i++) {
                brep |= on(bmask, i) << binds[i];
            }
            assert((amask & brep) == 0);
            ans[amask | brep] += a[amask] * b[brep];
        }
    }
    return ans;
}

void add_assign(A& a, const A& b) {
    for (int i = 0; i < (1 << maxn); i++) {
        a[i] += b[i];
    }
}

void solve() {
    int n, m;
    cin >> n >> m;
    bool arr[n][n] {};
    for (int i = 0; i < m; i++) {
        int u, v;
        cin >> u >> v;
        u--;
        v--;
        arr[u][v] = arr[v][u] = true;
    }
    for (int i = 0; i < n - 1; i++) {
        int u, v;
        cin >> u >> v;
        u--;
        v--;
        graph[u].push_back(v);
        graph[v].push_back(u);
    }
    dfs(0);
    A dp[n][n] {};
    for (int ti = n - 1; ti >= 0; ti--) {
        int u = ord[ti];
        for (int cx = 0; cx < n; cx++) {
            int cn = 1;
            A& cdp = dp[u][cx];
            cdp[1 << cx] = 1;
            for (auto& v : graph[u]) {
                A vdp {};
                for (int cv = 0; cv < n; cv++) {
                    if (!arr[cx][cv]) {
                        continue;
                    }
                    add_assign(vdp, dp[v][cv]);
                }
                cdp = merge(cn, cdp, siz[v], vdp);
                cn += siz[v];
            }
        }
    }
    long ans = 0;
    for (auto& a : dp[0]) {
        ans += accumulate(begin(a), end(a), long(0));
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    pcomp();
    solve();
}