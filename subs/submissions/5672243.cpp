#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
}

#ifdef DEBUG
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__)
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

template <typename T>
void sort_unique(vector<T>& arr) {
    sort(begin(arr), end(arr));
    arr.erase(unique(begin(arr), end(arr)), arr.end());
}

template <typename T>
int lower_bound(const vector<T>& arr, const T& x) {
    return int(lower_bound(begin(arr), end(arr), x) - arr.begin());
}

struct DS {
    vector<int> arr_w;
    vector<long> arr;

    DS(const vector<int>& arr_w) : arr_w(arr_w), arr(sz(arr_w)) {}

    void update(int l, int r, int x) {
        for (int i = l; i < r; i++) {
            arr[i] += x;
        }
    }

    int query_all(int kv) const {
        int ans = 0;
        for (int i = 0; i < sz(arr); i++) {
            if (arr[i] >= kv) {
                ans += arr_w[i];
            }
        }
        return ans;
    }
};

void solve() {
    int n, kv;
    cin >> n >> kv;
    vector<array<int, 5>> arr(n);
    for (auto& [xl, xr, yl, yr, w] : arr) {
        cin >> xl >> yl >> xr >> yr >> w;
    }

    vector<int> comp_y;
    for (auto& [_xl, _xr, yl, yr, _w] : arr) {
        comp_y.push_back(yl);
        comp_y.push_back(yr);
    }
    sort_unique(comp_y);

    for (auto& [_xl, _xr, yl, yr, _w] : arr) {
        yl = lower_bound(comp_y, yl);
        yr = lower_bound(comp_y, yr);
    }

    vector<int> arr_w(sz(comp_y) - 1);
    for (int i = 0; i < sz(comp_y) - 1; i++) {
        arr_w[i] = comp_y[i + 1] - comp_y[i];
    }

    map<int, vector<array<int, 3>>> mp;
    for (auto& [xl, xr, yl, yr, w] : arr) {
        mp[xl].push_back({yl, yr, +w});
        mp[xr].push_back({yl, yr, -w});
    }

    DS ds(arr_w);
    int prv = 0;
    long ans = 0;

    for (auto& [t, evts] : mp) {
        ans += long(t - prv) * ds.query_all(kv);
        prv = t;

        for (auto& [ql, qr, x] : evts) {
            ds.update(ql, qr, x);
        }
    }

    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}