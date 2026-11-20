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

map<vector<int>, long> memo;

constexpr int maxn = 35;

int n, m, arr[maxn][maxn];
vector<pair<int, int>> deps[maxn][maxn];

long dp(vector<int> inds) {
    auto [it, inserted] = memo.emplace(inds, 0);
    if (!inserted) {
        return it->second;
    }
    long& ans = it->second;
    for (int i = 0; i < n; i++) {
        if (inds[i]) {
            for (auto& [x, y] : deps[i][inds[i] - 1]) {
                if (inds[x] > y) {
                    goto loop;
                }
            }
            inds[i]--;
            ans = max(ans, arr[i][inds[i]] + dp(inds));
            inds[i]++;
        loop:;
        }
    }
    return ans;
}

void solve() {
    cin >> n >> m;
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < m; j++) {
            int kv;
            cin >> arr[i][j] >> kv;
            for (int k = 0; k < kv; k++) {
                int x, y;
                cin >> x >> y;
                deps[x][y].emplace_back(i, j);
            }
        }
    }
    cout << dp(vector<int>(n, m)) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}