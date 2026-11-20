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

constexpr int maxn = 105;

long arr[maxn], dist[maxn][maxn], memo[maxn][maxn][maxn];
vector<pair<int, long>> graph[maxn];

void pdfs(int src, int u, long d = 0) {
    dist[src][u] = d;
    for (auto& [v, w] : graph[u]) {
        pdfs(src, v, d + w);
    }
}

long dp(int u, int src, int k) {
    long& ans = memo[u][src][k];
    if (ans != -1) {
        return ans;
    }
    ans = 1e18;
    if (k) {
        ans = dp(u, u, k - 1);
    }
    long dpb[2][k + 1], *cdp = dpb[0], *pdp = dpb[1];
    fill(cdp, cdp + k + 1, 1e18);
    cdp[0] = 0;
    for (auto& [v, _] : graph[u]) {
        swap(cdp, pdp);
        fill(cdp, cdp + k + 1, 1e18);
        for (int i = 0; i <= k; i++) {
            for (int j = 0; i + j <= k; j++) {
                cdp[i + j] = min(cdp[i + j], pdp[i] + dp(v, src, j));
            }
        }
    }
    return ans = min(ans, arr[u] * dist[src][u] + *min_element(cdp, cdp + k + 1));
}

void solve() {
    int n, k;
    cin >> n >> k;
    for (int i = 1; i <= n; i++) {
        int x, p, w;
        cin >> x >> p >> w;
        graph[p].emplace_back(i, w);
        arr[i] = x;
    }
    for (int i = 0; i <= n; i++) {
        pdfs(i, i);
    }
    memset(memo, -1, sizeof(memo));
    cout << dp(0, 0, k) << endl;
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}