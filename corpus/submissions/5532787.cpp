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

constexpr int maxn = 1e4 + 5;

int n, cmask, memo[maxn][1 << 5];
vector<pair<int, int>> arr[maxn];

int eval(int ind, int mask) {
    int ans = 0;
    for (auto& [f, l] : arr[ind]) {
        dbg(f, l);
        ans += (mask & f) || (~mask & l);
    }
    return ans;
}

int eval5(int mask) {
    int ans = 0;
    for (int i = 4; i >= 0; i--) {
        ans += eval(i, mask >> i);
    }
    return ans;
}

int dp1(int ind, int mask);

int dp(int ind, int mask) {
    int& ans = memo[ind][mask];
    if (ans != -1) {
        return ans;
    }
    return ans = eval(ind, mask) + dp1(ind, mask);
}

int dp1(int ind, int mask) {
    assert(mask < 32);
    if (ind == 5) {
        return eval5(cmask | (mask << 5));
    }
    int nmask = (mask << 1) & 31;
    return max(dp(ind - 1, nmask), dp(ind - 1, nmask | 1));
}

void solve() {
    int m;
    cin >> n >> m;
    for (int it = 0; it < m; it++) {
        int ind, fn, ln;
        cin >> ind >> fn >> ln;
        ind--;
        auto to_mask = [&](int cn) -> int {
            int ans = 0;
            for (int it2 = 0; it2 < cn; it2++) {
                int x;
                cin >> x;
                x--;
                if (x < ind) {
                    x += n;
                }
                assert(x - ind < 5);
                ans |= 1 << (x - ind);
            }
            dbg(it, cn, ans);
            return ans;
        };
        int f = to_mask(fn), l = to_mask(ln);
        arr[ind].emplace_back(f, l);
    }
    int ans = 0;
    memset(memo, -1, sizeof(memo));
    for (cmask = 0; cmask < (1 << 5); cmask++) {
        memset(memo, -1, sizeof(memo));
        ans = max(ans, dp1(n, cmask));
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}