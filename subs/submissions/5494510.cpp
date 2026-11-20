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

constexpr int maxn = 1e3 + 5;

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

int n, arr[maxn][2];

map<array<int, 3>, int> memo;

int dp(int ind, int prv, int mask) {
    bool skip = true;
    int mxloc = 1e9;
    for (int i = 0; i <= 14; i++) {
        int nind = ind + i;
        if (nind < 0 || nind >= n) {
            continue;
        }
        if (!on(mask, i)) {
            if (i > 0 && skip) {
                return dp(ind + i, prv, mask >> i);
            }
            skip = false;
            mxloc = min(mxloc, nind + arr[nind][1]);
        }
    }
    dbg(ind, mask, mxloc);
    if (mxloc == 1e9) {
        return 0;
    }
    auto [it, inserted] = memo.insert({{ind, prv, mask}, 0});
    int& ans = it->second;
    if (!inserted) {
        return ans;
    }
    ans = 1e9;
    for (int i = ind; i <= mxloc; i++) {
        if (on(mask, i - ind)) {
            continue;
        }
        int ccost = 0;
        if (prv != -1) {
            ccost = arr[prv][0] ^ arr[i][0];
        }
        ans = min(ans, ccost + dp(ind, i, mask | (1 << (i - ind))));
    }
    return ans;
}

void solve() {
    cin >> n;
    for (int i = 0; i < n; i++) {
        cin >> arr[i][0] >> arr[i][1];
    }
    memo.clear();
    cout << dp(0, -1, 0) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}