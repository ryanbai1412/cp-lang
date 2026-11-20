#include "bits/extc++.h"

using namespace std;

template<typename T>
void dbgh(const T &t) {
    cerr << t << endl;
}

template<typename T, typename... U>
void dbgh(const T &t, const U &...u) {
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

constexpr int maxn = 1e5 + 2;

bool bad;
int ans, chs[maxn][2];

pair<int, int> dfs(int u, int d) {
    if (u == -1) {
        return {d, d};
    }
    auto [la, lb] = dfs(chs[u][0], d - 1);
    auto [ra, rb] = dfs(chs[u][1], d - 1);
    dbg(u, la, lb, ra, rb);
    if (lb <= ra) {
        return {la, rb};
    } else if (rb <= la) {
        ans += !(ra == rb && la == lb && rb == la);
        dbg(u, ra < la);
        return {ra, lb};
    }
    bad = true;
    return {-1, -1};
}

void solve() {
    int n;
    cin >> n;
    for (int i = 0; i < n; i++) {
        auto& [l, r] = chs[i];
        cin >> l >> r;
        l--;
        r--;
        if (l == -2) {
            l = -1;
        }
        if (r == -2) {
            r = -1;
        }
    }
    auto [cl, cr] = dfs(0, 0);
    if (bad || (cr - cl + 1) > 2) {
        cout << -1 << endl;
    } else {
        cout << ans << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}