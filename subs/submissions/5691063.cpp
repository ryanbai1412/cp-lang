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

namespace __gnu_pbds {

template <typename T>
using ost =
    tree<T, null_type, less<T>, rb_tree_tag, tree_order_statistics_node_update>;

}

using __gnu_pbds::ost;

struct MOst : ost<pair<int, int>> {
    using super = ost<pair<int, int>>;

    int id = 0;

    using super::super;

    void insert(int ind) {
        super::insert({ind, id++});
    }

    void erase_one(int ind) {
        auto it = super::lower_bound({ind, -1});
        if (it != super::end() && it->first == ind) {
            super::erase(it);
        }
    }

    int query_lt(int ind) const {
        return int(super::order_of_key({ind, -1}));
    }

    int query_ge(int ind) const {
        return sz(*this) - query_lt(ind);
    }
};

struct Vegetable {
    int w_each, w_first, c_init, c_spoil;
};

long solve(const vector<Vegetable>& arr, int kv, int max_t) {
    int n = sz(arr);

    auto c_left = [&](int ind, int t) -> int {
        return int(max(long(0), arr[ind].c_init - long(t) * arr[ind].c_spoil));
    };
    auto get_cost = [&](int ind, int used_cnt) -> int {
        int ans = arr[ind].w_each;
        if (!used_cnt) {
            ans += arr[ind].w_first;
        }
        return ans;
    };

    long ans = 0;
    int used[n] {};

    priority_queue<pair<long, int>> pq;

    vector<int> active[max_t + 1];
    for (int i = 0; i < n; i++) {
        int nw = 1e9;
        if (arr[i].c_spoil) {
            nw = arr[i].c_init / arr[i].c_spoil + 3;
        }
        active[min(max_t - 1, nw)].push_back(i);
    }

    for (int t = max_t - 1; t >= 0; t--) {
        for (auto& a : active[t]) {
            if (used[a] >= c_left(a, t)) {
                if (t) {
                    active[t - 1].push_back(a);
                }
                continue;
            }

            pq.emplace(get_cost(a, used[a]), a);
        }

        for (int it = 0; it < kv; it++) {
            if (!sz(pq)) {
                break;
            }

            auto [w, ind] = pq.top();
            pq.pop();

            ans += w;
            used[ind]++;

            if (used[ind] < c_left(ind, t)) {
                pq.emplace(get_cost(ind, used[ind]), ind);
            } else {
                if (arr[ind].c_init) {
                    if (t) {
                        active[t - 1].push_back(ind);
                    }
                }
            }
        }
    }

    return ans;
}

void solve() {
    int n, kv, q;
    cin >> n >> kv >> q;

    vector<Vegetable> arr(n);
    for (auto& [w_each, w_first, c_init, c_spoil] : arr) {
        cin >> w_each >> w_first >> c_init >> c_spoil;
    }

    while (q--) {
        int t;
        cin >> t;
        if (!t) {
            cout << 0 << endl;
            continue;
        }
        cout << solve(arr, kv, t) << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}