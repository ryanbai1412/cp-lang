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

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

struct DS {
    vector<pair<int, long>> arr;

    void push_front(long x) {
        arr.emplace(arr.begin(), -1e9, x);
    }

    void pop_back() {
        arr.pop_back();
    }

    void update_all(int x) {
        for (auto& [a, _b] : arr) {
            a = max(a, x);
        }
    }

    long query_all() const {
        long ans = 0;

        for (auto& [a, b] : arr) {
            ans = max(ans, a + b);
        }

        return ans;
    }
};

void solve() {
    int n, kv;
    cin >> n >> kv;

    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }

    long dp[n];

    for (int qr = n - 1; qr >= 0; qr -= kv) {
        // [ql, qr]
        int ql = max(0, qr - kv + 1);

        DS ds;
        for (int i = qr + kv; i > qr; i--) {
            if (i >= n) {
                ds.push_front(0);
            } else {
                ds.update_all(arr[i]);
                ds.push_front(dp[i]);
            }
        }

        for (int i = qr; i >= ql; i--) {
            ds.update_all(arr[i]);
            dp[i] = ds.query_all();
            ds.pop_back();
        }
    }

    cout << dp[0] << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}