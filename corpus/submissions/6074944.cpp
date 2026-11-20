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

struct Query {
    int xl, xr, yl, yr, mul;
    ref_w<long> ans;

    Query transposed() const {
        return {yl, yr, xl, xr, mul, ans};
    }
};

void solve_queries(const vector<int>& arr, const vector<Query>& queries) {
    int n = sz(arr), q = sz(queries);

    for (auto& a : queries) {
        int ans = 0;

        for (int i = a.xl; i < a.xr; i++) {
            for (int j = i + 1; j < a.xr; j++) {
                ans += a.yl <= arr[j] && arr[j] < arr[i] && arr[i] < a.yr;
            }
        }

        dbg(ans, a.mul, a.xl, a.xr, a.yl, a.yr);
        a.ans += ans * a.mul;
    }
}

void solve_count(const vector<int>& arr, const vector<Query>& queries) {
    int n = sz(arr), q = sz(queries);

    for (auto& a : queries) {
        int ans = 0;

        for (int i = a.xl; i < a.xr; i++) {
            ans += a.yl <= arr[i] && arr[i] < a.yr;
        }

        a.ans *= ans;
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

    vector<long> qans(q), count_ans(q, 1);

    vector<Query> query_norm, query_trans, query_count;

    auto query = [&](Query cq) -> void {
        if (cq.xl == cq.xr || cq.yl == cq.yr) {
            return;
        }

        if (cq.yl == 0) {
            query_norm.push_back(cq);
        } else {
            query_trans.push_back(cq.transposed());
        }
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
        query_count.push_back(Query {xl, xr, yl, yr, +1, count_ans[qi]});
        query_count.push_back(Query {xr, n, 0, yl, +1, count_ans[qi]});
    }

    solve_queries(arr, query_norm);
    solve_queries([&]() {
        vector<int> t_arr(n);

        for (int i = 0; i < n; i++) {
            t_arr[arr[i]] = i;
        }

        return t_arr;
    }(), query_trans);
    solve_count(arr, query_count);

    for (int qi = 0; qi < q; qi++) {
        long ans = qans[qi] + count_ans[qi];
        dbg(qans[qi], count_ans[qi]);
        cout << ans << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}