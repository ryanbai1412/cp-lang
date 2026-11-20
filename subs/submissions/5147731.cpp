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

struct DS {
    vector<pair<long, int>> arr;

    void insert(long k, int v) {
        arr.emplace_back(k, v);
    }

    int query(long x) {
        int ans = 0;
        for (auto& [k, v] : arr) {
            if (k <= x) {
                ans = max(ans, v);
            }
        }
        return ans;
    }
};

long sq(long x) {
    return x * x;
}

void solve(const vector<int>& arr) {
    int n = sz(arr);
    long psum[n + 1];
    psum[0] = 0;
    for (int i = 0; i < n; i++) {
        psum[i + 1] = psum[i] + arr[i];
    }
    DS ds;
    pair<long, long> dp[n + 1];
    dp[0] = {};
    for (int i = 1; i <= n; i++) {
        {
            int j = ds.query(psum[i]);
            dp[i] = {dp[j].first - 2 * psum[i] * psum[j], -psum[j]};
        }
        dp[i].first += 2 * sq(psum[i]);
        dp[i].second += 2 * psum[i];
        ds.insert(dp[i].second, i);
    }
    cout << dp[n].first - sq(psum[n]) << endl;
}

void solve() {
    int n, tc_type;
    cin >> n >> tc_type;
    vector<int> arr(n);
    for (auto& a : arr) {
        cin >> a;
    }
    solve(arr);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}