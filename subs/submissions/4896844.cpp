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
    int m;
    cin >> m;
    string arr[m];
    for (auto& a : arr) {
        cin >> a;
    }
    string s;
    cin >> s;
    int n = sz(s), inds[m][100];
    memset(inds, -1, sizeof(inds));
    vector<pair<int, int>> arri[256];
    for (int i = 0; i < m; i++) {
        for (int j = sz(arr[i]) - 1; j >= 0; j--) {
            arri[int(arr[i][j])].emplace_back(i, j);
        }
    }
    int dp[n];
    pair<int, int> opt[n];
    auto get = [&](int i) -> int {
        return i >= 0 ? dp[i] : 0;
    };
    for (int i = 0; i < sz(s); i++) {
        dp[i] = get(i - 1);
        opt[i] = {-1, i - 1};
        for (auto& [a, b] : arri[int(s[i])]) {
            if (!b) {
                inds[a][b] = i;
            } else {
                inds[a][b] = inds[a][b - 1];
            }
            if (b == sz(arr[a]) - 1 && inds[a][b] != -1 && i - inds[a][b] <= 1000) {
                int cur = sz(arr[a]) + get(inds[a][b] - 1);
                if (cur > dp[i]) {
                    dp[i] = cur;
                    opt[i] = {a, inds[a][b] - 1}; 
                }
            }
        }
    }
    cout << dp[n - 1] << endl;
    int i = n - 1;
    while (i >= 0) {
        auto [cx, ni] = opt[i];
        if (cx == -1) {
            i = ni;
            continue;
        }
        cout << cx + 1 << " " << ni + 2 << " " << i + 1 << endl;
        i = ni;
    }
    cout << "A" << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}