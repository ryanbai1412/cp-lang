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

struct sint {
    static constexpr long inf = 2e18;

    long x;

    sint() : x(0) {}
    sint(long x) : x(min(x, inf)) {}

    friend ostream& operator<<(ostream& out, const sint& s) {
        return out << s.x;
    }

    sint& operator+=(const sint& s) {
        x = min(inf, x + s.x);
        return *this;
    }
    sint& operator-=(const sint& s) {
        x = min(inf, x - s.x);
        return *this;
    }
    sint& operator*=(const sint& s) {
        if (x >= inf / s.x) {
            x = inf;
        } else {
            x *= s.x;
        }
        return *this;
    }

    friend sint operator+(sint a, const sint& b) {
        return a += b;
    }
    friend sint operator-(sint a, const sint& b) {
        return a -= b;
    }
    friend sint operator*(sint a, const sint& b) {
        return a *= b;
    }

    bool operator<(const sint& s) const {
        return x < s.x;
    }
    bool operator>(const sint& s) const {
        return x > s.x;
    }

    sint abs() const {
        return max(x, -x);
    }
};

int p;

struct Line {
    sint k, c;
    int data;

    sint operator()(sint x) const {
        x = (k + x).abs();
        sint cp = 1;
        for (int i = 0; i < p; i++) {
            cp *= x;
        }
        return cp + c;
    }
};

struct DS {
    vector<Line> arr;

    void insert(const Line& x) {
        arr.push_back(x);
    }

    pair<sint, int> query(int x) const {
        pair<sint, int> ans {long(2e18), -1};
        for (auto& a : arr) {
            ans = min(ans, pair<sint, int> {a(x), a.data});
        }
        return ans;
    }
};

void solve() {
    int n, kv;
    cin >> n >> kv >> p;
    string sarr[n];
    for (auto& a : sarr) {
        while (!sz(a)) {
            getline(cin, a);
        }
    }
    int arr[n];
    for (int i = 0; i < n; i++) {
        arr[i] = sz(sarr[i]);
    }
    int psum[n + 1];
    psum[0] = 0;
    partial_sum(arr, arr + n, psum + 1);
    DS ds;
    sint dp[n];
    int opt[n];
    ds.insert({kv - (psum[n] + n), 0, n});
    for (int i = n - 1; i >= 0; i--) {
        auto [cans, copt] = ds.query(psum[i] + i + 1);
        dp[i] = cans;
        opt[i] = copt;
        ds.insert({kv - (psum[i] + i), cans, i});
    }
    if (dp[0] > long(1e18)) {
        cout << "Too hard to arrange" << endl;
    } else {
        cout << dp[0] << endl;
        for (int i = 0; i < n; ) {
            int j = opt[i];
            for (; i < j; i++) {
                cout << sarr[i] << " \n"[i == j - 1];
            }
        }
    }
    cout << string(20, '-') << endl;
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