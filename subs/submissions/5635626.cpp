#pragma GCC optimize("Ofast")
#pragma GCC target("avx2")

#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
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

using u64 = uint64_t;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

constexpr int maxn = 20;

bool arr[maxn][maxn];
vector<int> c_nodes, graph[maxn], ord;

void pdfs(int u) {
    for (auto& v : graph[u]) {
        graph[v].erase(find(begin(graph[v]), end(graph[v]), u));
        pdfs(v);
    }

    ord.push_back(u);
}

void solve() {
    int n, m;
    cin >> n >> m;
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

    pdfs(0);

    auto pow_n1 = [&](int x) -> u64 {
        if (x & 1) {
            return -1;
        } else {
            return 1;
        }
    };

    u64 ans = 0;

    for (int mask = 1; mask < (1 << n); mask++) {
        c_nodes.clear();
        for (int i = 0; i < n; i++) {
            if (on(mask, i)) {
                c_nodes.push_back(i);
            }
        }

        int cm = sz(c_nodes);

        vector<int> cgraph[cm];

        for (int i = 0; i < cm; i++) {
            for (int j = 0; j < cm; j++) {
                if (arr[c_nodes[i]][c_nodes[j]]) {
                    cgraph[i].push_back(j);
                }
            }
        }

        u64 dp[n][cm];

        for (auto& u : ord) {
            for (int v1 = 0; v1 < cm; v1++) {
                u64& cdp = dp[u][v1];

                cdp = 1;

                for (auto& v : graph[u]) {
                    u64 cmul = 0;

                    for (int v2 = 0; v2 < cm; v2++) {
                        cmul += arr[c_nodes[v1]][c_nodes[v2]] * dp[v][v2];
                    }

                    cdp *= cmul;
                }
            }
        }
        ans += pow_n1(__builtin_popcount(mask) + n) *
               accumulate(dp[0], dp[0] + cm, u64(0));
    }

    cout << long(ans) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}