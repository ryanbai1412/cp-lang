#ifndef LOCAL
#pragma GCC optimize("Ofast")
#pragma GCC target("avx2")
#endif

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

template <typename T>
using ref_w = reference_wrapper<T>;

template <typename T>
ostream& operator<<(ostream& out, const vector<T>& arr) {
    out << "[";
    for (int i = 0; i < sz(arr); i++) {
        if (i) {
            out << ", ";
        }
        out << arr[i];
    }
    return out << "]";
}

vector<int> transposed(vector<int> arr) {
    int n = sz(arr);

    vector<int> ans(n);

    for (int i = 0; i < n; i++) {
        ans[arr[i]] = i;
    }

    return ans;
}

template <typename T>
int lbi(const vector<T>& arr, const T& x) {
    return int(lower_bound(begin(arr), end(arr), x) - arr.begin());
}

template <typename T>
struct BIT {
    int n;
    vector<T> v;

    BIT(int n) : n(n), v(n + 1) {}

    void update(int ind, T x) {
        ind++;
        while (ind <= n) {
            v[ind] += x;
            ind += ind & -ind;
        }
    }

    T query(int ind) const {
        T ans = 0;
        while (ind > 0) {
            ans += v[ind];
            ind -= ind & -ind;
        }
        return ans;
    }

    T query(int l, int r) const {
        return query(r) - query(l);
    }
};

template <typename T>
struct DS1 {
    static constexpr int BSIZE = 512;

    int n;
    vector<T> arr, block_sums;

    DS1(int n) : n(n), arr(n), block_sums(n / BSIZE + 1) {}

    void update(int ind, T x) {
        arr[ind] += x;
        block_sums[ind / BSIZE] += x;
    }

    T query(int ind) const {
        int mb = ind / BSIZE;

        T ans = 0;

        for (int i = 0; i < mb; i++) {
            ans += block_sums[i];
        }
        for (int i = mb * BSIZE; i < ind; i++) {
            ans += arr[i];
        }

        return ans;
    }
};

struct Query {
    int xl, xr, yl, yr, mul;
    ref_w<long> ans;
};

int solve_bf(const vector<int>& arr, const Query& query) {
    int ans = 0;

    auto& a = query;
    for (int i = a.xl; i < a.xr; i++) {
        for (int j = i + 1; j < a.xr; j++) {
            ans += a.yl <= arr[j] && arr[j] < arr[i] && arr[i] < a.yr;
        }
    }

    return ans;
}

void solve_queries(const vector<int>& arr, const vector<Query>& queries) {
    constexpr int BSIZE = 400;

    int n = sz(arr), q = sz(queries);

    auto evt_arr = transposed(arr);

    vector<int> r_down(n);

    {
        BIT<int> bit(n);
        for (int i = n - 1; i >= 0; i--) {
            r_down[i] = bit.query(arr[i]);
            bit.update(arr[i], +1);
        }
    }

    vector<int> evt_queries[n / BSIZE + 1];
    {
        BIT<int> bit(n);
        for (int qi = 0; qi < q; qi++) {
            auto& a = queries[qi];
            assert(!a.yl);

            if (a.xr - a.xl <= 2 * BSIZE) {
                dbg(a.xl, a.xr, a.yl, a.yr);
                long ans = 0;

                for (int i = a.xl; i < a.xr; i++) {
                    if (!(a.yl <= arr[i] && arr[i] < a.yr)) {
                        continue;
                    }

                    ans += bit.query(arr[i], n);
                    bit.update(arr[i], +1);
                }

                for (int i = a.xl; i < a.xr; i++) {
                    if (!(a.yl <= arr[i] && arr[i] < a.yr)) {
                        continue;
                    }

                    bit.update(arr[i], -1);
                }

                a.ans += ans * a.mul;
                continue;
            }

            evt_queries[a.xr / BSIZE].push_back(qi);
        }
    }

    vector<int> l_q[n];

    for (int bi = 0; bi <= n / BSIZE; bi++) {
        int bl = bi * BSIZE, br = min(n, bl + BSIZE), bsz = br - bl;

        vector<int> r_cnt(n);
        {
            for (int i = bl; i < n; i++) {
                r_cnt[arr[i]]++;
            }
            for (int i = 0; i + 1 < n; i++) {
                r_cnt[i + 1] += r_cnt[i];
            }
        }

        // lower_bound
        vector<int> comp(n);
        vector<int> c_right(arr.begin() + bl, arr.begin() + br);
        sort(begin(c_right), end(c_right));

        for (int i = 0, j = 0; i < n; i++) {
            while (j < sz(c_right) && c_right[j] < i) {
                j++;
            }

            comp[evt_arr[i]] = j;
            assert(j == lbi(c_right, i));
        }

        vector<int> r_in[bsz];
        {
            for (int i = bl; i < br; i++) {
                dbg(arr[i], comp[i], bl, br, sz(c_right));
                int ri = i - bl;

                r_in[ri].resize(bsz + 1);

                for (int j = bl; j < i; j++) {
                    if (arr[j] > arr[i]) {
                        r_in[ri][comp[j] + 1]++;
                    }
                }
                for (int j = 0; j < bsz; j++) {
                    r_in[ri][j + 1] += r_in[ri][j];
                }

                if (ri) {
                    for (int j = 0; j <= bsz; j++) {
                        r_in[ri][j] += r_in[ri - 1][j];
                    }
                }
            }
        }

        for (auto& a : l_q) {
            a.clear();
        }
        for (auto& a : evt_queries[bi]) {
            assert(queries[a].xl < bl);
            l_q[queries[a].xl].push_back(a);
        }

        DS1<long> mid_ds1(n);
        BIT<int> mid_cnt(n);
        for (int ql = bl - 1; ql >= 0; ql--) {
            {
                mid_ds1.update(arr[ql], r_down[ql] - r_cnt[arr[ql]]);
                mid_cnt.update(arr[ql], +1);
            }

            for (auto& qi : l_q[ql]) {
                auto& a = queries[qi];

                int yr = a.yr;
                long ans = mid_ds1.query(yr), base = mid_cnt.query(a.yr);

                for (int i = bl; i < a.xr; i++) {
                    if (arr[i] >= a.yr) {
                        continue;
                    }

                    ans += base - mid_cnt.query(arr[i]);
                    // mid_cnt.update(arr[i], +1);
                }
                if (a.xr > bl) {
                    dbg(a.xl, a.xr, bl, a.yr, c_right, arr);
                    dbg(r_in[1][2]);
                    int fast_ans =
                        r_in[a.xr - bl - 1][lbi(c_right, a.yr)];

                    // BIT<int> cbit(n);
                    // int xans = 0;
                    // for (int i = bl; i < a.xr; i++) {
                    //     if (arr[i] >= a.yr) {
                    //         continue;
                    //     }
                    //
                    //     xans += cbit.query(arr[i], a.yr);
                    //     cbit.update(arr[i], +1);
                    // }

                    ans += fast_ans;
                    // assert(xans == fast_ans);
                    // dbg(xans, fast_ans);
                }
                // for (int i = bl; i < a.xr; i++) {
                //     if (arr[i] >= a.yr) {
                //         continue;
                //     }
                //
                //     mid_cnt.update(arr[i], -1);
                // }

                dbg(a.xl, a.xr, a.yl, a.yr, ans, a.mul);
                dbg(ans, solve_bf(arr, a));
                a.ans += ans * a.mul;
            }
        }
    }

    // for (auto& a : queries) {
    //     int ans = 0;
    //
    //     for (int i = a.xl; i < a.xr; i++) {
    //         for (int j = i + 1; j < a.xr; j++) {
    //             ans += a.yl <= arr[j] && arr[j] < arr[i] && arr[i] < a.yr;
    //         }
    //     }
    //
    //     dbg(ans, a.mul, a.xl, a.xr, a.yl, a.yr);
    //     a.ans += ans * a.mul;
    // }
}

void solve_count(const vector<int>& arr, const vector<Query>& queries) {
    int n = sz(arr), q = sz(queries);

    auto evt_arr = transposed(arr);
    vector<int> evt_query[n + 1];

    for (int qi = 0; qi < q; qi++) {
        evt_query[queries[qi].yr].push_back(qi);
    }

    BIT<int> bit(n);
    for (int i = 0; i <= n; i++) {
        for (auto& qi : evt_query[i]) {
            auto& a = queries[qi];
            a.ans += bit.query(a.xl, a.xr) * a.mul;
        }

        if (i < n) {
            bit.update(evt_arr[i], +1);
        }
    }
}

void solve() {
    int n, q;
    cin >> n >> q;

    vector<int> arr(n);
    for (auto& a : arr) {
        cin >> a;
        a = n - a;
    }

    vector<long> qans(q);
    vector<pair<long, long>> count_ans(q);

    vector<Query> query_norm, query_trans, query_count;

    auto query = [&](Query cq) -> void {
        if (cq.xl == cq.xr || cq.yl == cq.yr) {
            return;
        }

        if (cq.yl == 0) {
            query_norm.push_back(cq);
        } else {
            assert(n == cq.xr);
            dbg(cq.xl, cq.xr, cq.yl, cq.yr);
            query_trans.push_back(
                {n - cq.yr, n - cq.yl, n - cq.xr, n - cq.xl, cq.mul, cq.ans});
        }
    };
    auto do_query_count = [&](Query cq) -> void {
        query_count.push_back({cq.xl, cq.xr, 0, cq.yr, +cq.mul, cq.ans});
        query_count.push_back({cq.xl, cq.xr, 0, cq.yl, -cq.mul, cq.ans});
    };

    for (int qi = 0; qi < q; qi++) {
        auto do_query = [&](int xl, int xr, int yl, int yr, int mul) -> void {
            dbg(xl, xr, yl, yr, mul);
            query({xl, xr, yl, yr, mul, qans[qi]});
        };

        int xl, xr, yl, yr;
        cin >> xl >> xr >> yl >> yr;
        xl--;
        yl = n - yl;
        yr = n - yr;
        swap(yl, yr);
        yr++;

        do_query(xl, n, yl, yr, +1);
        do_query(xl, xr, 0, yr, +1);
        do_query(xr, n, 0, yr, +1);
        do_query(xl, n, 0, yl, +1);
        do_query(xl, n, 0, yr, -1);
        do_query(xr, n, yl, yr, -1);
        do_query(xl, xr, 0, yl, -1);
        do_query(xr, n, 0, yl, -1);
        do_query_count(Query {xl, xr, yl, yr, +1, count_ans[qi].first});
        do_query_count(Query {xr, n, 0, yl, +1, count_ans[qi].second});
    }

    solve_queries(arr, query_norm);
    solve_queries([&]() {
        vector<int> n_arr(n);

        for (int i = 0; i < n; i++) {
            n_arr[n - arr[i] - 1] = n - i - 1;
        }

        return n_arr;
    }(), query_trans);
    solve_count(arr, query_count);

    for (int qi = 0; qi < q; qi++) {
        long ans = qans[qi] + count_ans[qi].first * count_ans[qi].second;
        dbg(qans[qi], ans - qans[qi]);
        cout << ans << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}