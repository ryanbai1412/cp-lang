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

template <typename T>
T reversed(T t) {
    reverse(begin(t), end(t));
    return t;
}

struct PSA {
    vector<int> psum;
    
    PSA(int n): psum(n + 1) {}
    
    void update(int l, int r, int x) {
        psum[l] += x;
        psum[r + 1] -= x;
    }
    
    int query() const {
        vector<int> ans(sz(psum));
        partial_sum(begin(psum), end(psum), ans.begin());
        return *max_element(begin(ans), end(ans));
    }
};

int solve(const vector<int>& arr) {
    int n = sz(arr);
    auto check = [&](int x) -> bool {
        PSA ds(n);
        auto upd = [&](int l, int r) -> void {
            l = max(0, l);
            r = min(n - 1, r);
            if (l <= r) {
                ds.update(l, r, 1);
            }
        };
        for (int i = 0; i < n; i++) {
            int xl = i - arr[i] - x, xr = i - arr[i] + x;
            while (xl < 0) {
                xl += n;
                xr += n;
            }
            upd(xl, xr);
            upd(xl - n, min(xl - 1, xr - n));
        }
        return ds.query() == n;
    };
    int l = 0, r = n;
    while (l < r) {
        int mid = (l + r) / 2;
        if (check(mid)) {
            r = mid;
        } else {
            l = mid + 1;
        }
    }
    return l;
}

void solve() {
    int n;
    cin >> n;
    vector<int> arr(n);
    for (auto& a : arr) {
        cin >> a;
        a--;
    }
    cout << min(solve(arr), solve(reversed(arr))) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}