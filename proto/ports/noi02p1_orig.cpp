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
#define sz(x) int(size(x))

constexpr int n = 3e4 + 5;

int p[n], dist[n];

pair<int, int> find(int u) {
    if (p[u] < 0) {
        return {u, 0};
    }
    auto [v, d] = find(p[u]);
    p[u] = v;
    dist[u] += d;
    return {p[u], dist[u]};
}

void solve() {
    int q;
    cin >> q;
    memset(p, -1, sizeof(p));
    while (q--) {
        char t;
        int u, v;
        cin >> t >> u >> v;
        u--;
        v--;
        auto [pu, du] = find(u);
        auto [pv, dv] = find(v);
        if (t == 'M') {
            if (pu != pv) {
                dist[pu] = -p[pv];
                p[pv] += p[pu];
                p[pu] = pv;
            }
        } else {
            if (pu == pv) {
                cout << abs(du - dv) - 1 << endl;
            } else {
                cout << -1 << endl;
            }
        }
    }
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}