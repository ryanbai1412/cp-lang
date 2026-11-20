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
    int n;
    vector<int> arr, v;

    DS(const vector<int>& arr) : n(sz(arr)), arr(arr), v(n) {}

    void update(int ind) {
        for (int i = 0; i < ind; i++) {
            v[i]++;
        }
    }

    long query() const {
        long mx = 0;
        for (int i = 0; i < n; i++) {
            mx = max(mx, long(arr[i]) * v[i]);
        }
        return mx;
    }
};

void solve() {
    int n;
    cin >> n;
    vector<pair<int, int>> arr(n);
    for (auto& [x, y] : arr) {
        cin >> x >> y;
    }

    map<int, vector<int>> evts;
    for (int i = 0; i < n; i++) {
        evts[arr[i].first].push_back(i);
    }

    vector<int> ys = [&]() {
        vector<int> res;
        for (auto& [_x, y] : arr) {
            res.push_back(y);
        }
        sort(begin(res), end(res));
        res.erase(unique(begin(res), end(res)), end(res));

        return res;
    }();

    DS ds(ys);
    int gt_rem = n;
    long ans = 0;

    for (auto& [k, ev] : evts) {
        ans = max(ans, gt_rem * long(k) + ds.query());

        for (auto& i : ev) {
            ds.update(int(upper_bound(begin(ys), end(ys), arr[i].second) -
                          begin(ys)));
        }
        gt_rem -= sz(ev);
    }

    ans = max(ans, ds.query());

    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}