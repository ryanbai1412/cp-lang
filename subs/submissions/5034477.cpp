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

int memo[maxn], opt[maxn];

int dp(int n) {
    if (n <= 1) {
        return 0;
    }
    int& ans = memo[n];
    if (ans != -1) {
        return ans;
    }
    ans = 1e9;
    for (int i = 0; i < n; i++) {
        int cx = max(i, dp(n - i - 1)) + 1;
        if (cx <= ans) {
            ans = cx;
            opt[n] = i;
        }
    }
    return ans;
}

void solve(int l, int r) {
    if (l == r) {
        cout << "Decide " << l << endl;
        return;
    }
    dp(r - l + 1);
    int i = opt[r - l + 1];
    cout << "Drop " << l + i + 1 << endl;
    int res;
    cin >> res;
    if (res) {
        for (int j = l + 1; j < l + i + 1; j++) {
            cout << "Drop " << j << endl;
            cin >> res;
            if (res) {
                cout << "Decide " << j << endl;
                return;
            }
        }
        cout << "Decide " << i << endl;
        return;
    } else {
        solve(l + i + 2, r);
    }
}

void solve() {
    cout << "GetN" << endl;
    int n;
    cin >> n;
    memset(memo, -1, sizeof(memo));
    solve(0, n);
}

int main() {
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}