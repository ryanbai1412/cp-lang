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

void solve() {
    int n;
    cin >> n;
    vector<int> graph[n];
    for (int i = 0; i < n; i++) {
        int m;
        cin >> m;
        for (int j = 0; j < m; j++) {
            int p;
            cin >> p;
            p--;
            graph[p].push_back(j);
        }
    }
    int ans = 1e9;
    for (int i = 0; i < n; i++) {
        queue<int> q;
        vector<int> ord;
        int par[n];
        memset(par, -1, sizeof(par));
        auto push = [&](int u, int p) -> void {
            if (par[u] != -1) {
                return;
            }
            par[u] = p;
            q.push(u);
        };
        push(i, -2);
        while (sz(q)) {
            int u = q.front();
            q.pop();
            ord.push_back(u);
            for (auto& v : graph[u]) {
                push(v, u);
            }
        }
        if (sz(ord) != n) {
            continue;
        }
        int dp[n] {};
        reverse(begin(ord), end(ord));
        for (auto& a : ord) {
            dp[a]++;
            if (par[a] >= 0) {
                dp[par[a]] += dp[a];
            }
        }
        ans = min(ans, accumulate(dp, dp + n, 0));
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}