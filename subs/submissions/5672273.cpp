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

struct Block {
    int n;
    long add;
    vector<long> vals;
    vector<int> ssum;

    Block(long add, vector<pair<long, int>> arr)
        : n(sz(arr)), add(add), vals(n), ssum(n + 1) {
        sort(begin(arr), end(arr));
        for (int i = 0; i < n; i++) {
            vals[i] = arr[i].first;
        }
        for (int i = n - 1; i >= 0; i--) {
            ssum[i] = ssum[i + 1] + arr[i].second;
        }
    }

    void range_add(int x) {
        add += x;
    }

    int query_all(int kv) const {
        return ssum[lower_bound(vals, kv - add)];
    }
};

struct DS {
    static constexpr int BSIZE = 400;

    int n;
    vector<pair<long, int>> arr;
    vector<Block> blocks;

    DS(const vector<int>& arr_w) : n(sz(arr_w)), arr(n) {
        for (int i = 0; i < n; i++) {
            arr[i].second = arr_w[i];
        }

        for (int bql = 0; bql < n; bql += BSIZE) {
            int bqr = min(n, bql + BSIZE);

            blocks.emplace_back(0, vector<pair<long, int>>(arr.begin() + bql,
                                                           arr.begin() + bqr));
        }
    }

    void build(int bind) {
        int bql = bind * BSIZE, bqr = min(n, bql + BSIZE);

        blocks[bind] = Block(
            blocks[bind].add,
            vector<pair<long, int>>(arr.begin() + bql, arr.begin() + bqr));
    }

    void update(int l, int r, int x) {
        r--;
        if (l > r) {
            return;
        }

        int l_block = l / BSIZE, r_block = r / BSIZE;

        if (l_block + 1 >= r_block) {
            for (int i = l; i <= r; i++) {
                arr[i].first += x;
            }
            for (int i = l_block; i <= r_block; i++) {
                build(i);
            }
            return;
        }

        for (int i = l; i < (l_block + 1) * BSIZE; i++) {
            arr[i].first += x;
        }
        for (int i = r_block * BSIZE; i <= r; i++) {
            arr[i].first += x;
        }
        build(l_block);
        build(r_block);

        for (int i = l_block + 1; i < r_block; i++) {
            blocks[i].range_add(x);
        }
    }

    int query_all(int kv) const {
        int ans = 0;
        for (auto& a : blocks) {
            ans += a.query_all(kv);
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