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

constexpr int maxn = 20;

int n, trans[maxn][maxn], memo[1 << 20];
string arr[maxn];

int dp(int i, vector<int> vis) {
    int key = i;
    for (auto& a : vis) {
        key = key * 3 + a;
    }
    int& ans = memo[key];
    if (ans != -1) {
        return ans;
    }
    ans = 0;
    for (int j = 0; j < n; j++) {
        if (vis[j] < 2 && trans[i][j] != -1) {
            vis[j]++;
            ans = max(ans, trans[i][j] + dp(j, vis));
            vis[j]--;
        }
    }
    return ans;
}

void solve() {
    cin >> n;
    for (int i = 0; i <= n; i++) {
        cin >> arr[i];
    }
    memset(trans, -1, sizeof(trans));
    for (int i = 0; i <= n; i++) {
        for (int j = 0; j <= n; j++) {
            for (int k = 1; k < min(sz(arr[i]), sz(arr[j])) + (i == n); k++) {
                if (arr[i].substr(sz(arr[i]) - k) == arr[j].substr(0, k)) {
                    trans[i][j] = max(trans[i][j], sz(arr[j]) - k);
                }
            }
        }
    }
    memset(memo, -1, sizeof(memo));
    cout << sz(arr[n]) + dp(n, vector<int>(n, 0)) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}