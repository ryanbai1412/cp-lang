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

template <typename Cb>
struct CmpByKey {
    Cb cb;

    CmpByKey(Cb cb) : cb(cb) {}

    template <typename T>
    bool operator()(const T& a, const T& b) const {
        return cb(a) < cb(b);
    }
};

struct Block {
    int n, ql, qr;
    long add;
    vector<pair<long, int>>* main_arr_ptr;
    vector<long> vals;
    vector<int> ssum, ord;

    Block(long add, vector<pair<long, int>>& main_arr, int ql, int qr)
        : n(qr - ql),
          ql(ql),
          qr(qr),
          add(add),
          main_arr_ptr(&main_arr),
          vals(n),
          ssum(n + 1),
          ord(n) {
        iota(begin(ord), end(ord), ql);
        sort(begin(ord), end(ord),
             CmpByKey([&](int u) -> int { return main_arr[u].first; }));

        build_with_ord();
    }

    void build_with_ord() {
        const auto& main_arr = *main_arr_ptr;

        for (int i = 0; i < n; i++) {
            vals[i] = main_arr[ord[i]].first;
        }
        for (int i = n - 1; i >= 0; i--) {
            ssum[i] = ssum[i + 1] + main_arr[ord[i]].second;
        }
    }

    void range_add(int x) {
        add += x;
    }

    void pref_add(int ind, int x) {
        auto& main_arr = *main_arr_ptr;
        for (int i = ql; i <= ql + ind; i++) {
            main_arr[i].first += x;
        }

        int ord_out[n];

        merge(ord.begin(), ord.begin() + ind + 1, ord.begin() + ind + 1,
              ord.end(), ord_out,
              CmpByKey([&](int u) -> int { return main_arr[u].first; }));

        copy(ord_out, ord_out + n, ord.begin());

        build_with_ord();
    }

    void suff_add(int ind, int x) {
        auto& main_arr = *main_arr_ptr;
        for (int i = ql + ind; i < qr; i++) {
            main_arr[i].first += x;
        }

        int ord_out[n];

        merge(ord.begin(), ord.begin() + ind, ord.begin() + ind, ord.end(),
              ord_out,
              CmpByKey([&](int u) -> int { return main_arr[u].first; }));

        copy(ord_out, ord_out + n, ord.begin());

        build_with_ord();
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

            blocks.emplace_back(0, arr, bql, bqr);
        }
    }

    void build(int bind) {
        int bql = bind * BSIZE, bqr = min(n, bql + BSIZE);

        blocks[bind] = Block(blocks[bind].add, arr, bql, bqr);
    }

    void update(int l, int r, int x) {
        r--;
        if (l > r) {
            return;
        }

        int l_block = l / BSIZE, r_block = r / BSIZE;

        // if (l_block == r_block) {
        //     blocks[l_block].pref_add(r - l_block * BSIZE, x);
        //     if (l > l_block * BSIZE) {
        //         blocks[l_block].pref_add(l - 1 - l_block * BSIZE, -x);
        //     }
        //     return;
        // }
        if (l_block + 1 >= r_block) {
            // blocks[l_block].suff_add(l - l_bl)
            for (int i = l; i <= r; i++) {
                arr[i].first += x;
            }
            for (int i = l_block; i <= r_block; i++) {
                build(i);
            }
            return;
        }

        // blocks[l_block].suff_add(l - l_block * BSIZE, x);
        for (int i = l; i < (l_block + 1) * BSIZE; i++) {
            arr[i].first += x;
        }
        build(l_block);
        // blocks[r_block].pref_add(r - r_block * BSIZE, x);
        for (int i = r_block * BSIZE; i <= r; i++) {
            arr[i].first += x;
        }
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
    assert(1 <= n && n <= 2e5);
    assert(1 <= kv && kv <= 1e9);

    vector<array<int, 5>> arr(n);
    for (auto& [xl, xr, yl, yr, w] : arr) {
        cin >> xl >> yl >> xr >> yr >> w;
        assert(0 <= xl && xl < xr && xr <= int(1e9));
        assert(0 < yl && yl < yr && yr <= int(1e9));
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