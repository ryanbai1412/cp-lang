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
#define sz(x) int((x).size())

template <typename T>
int q_lb(const vector<T>& arr, const T& x) {
    return int(lower_bound(begin(arr), end(arr), x) - arr.begin());
}

struct Node {
    int lc, rc;
} heap[10 << 20];

int n, h_ind = 1;

void alloc(int& o) {
    heap[h_ind] = heap[o];
    o = h_ind++;
}

void insert(int& o, int l, int r, int ind) {
    alloc(o);
    if (l == r) {
        return;
    }
    int mid = (l + r) / 2;
    if (ind <= mid) {
        insert(heap[o].lc, l, mid, ind);
    } else {
        insert(heap[o].rc, mid + 1, r, ind);
    }
}

int insert(int o, int ind) {
    insert(o, 0, n - 1, ind);
    return o;
}

void update_clear(int& o, int l, int r, int ql) {
    if (!o) {
        return;
    } else if (ql <= l) {
        o = 0;
        return;
    }
    alloc(o);
    int mid = (l + r) / 2;
    update_clear(heap[o].rc, mid + 1, r, ql);
    if (ql <= mid) {
        update_clear(heap[o].lc, l, mid, ql);
    }

    if (!heap[o].lc && !heap[o].rc) {
        o = 0;
        return;
    }
}

int update_clear(int o, int ql) {
    update_clear(o, 0, n - 1, ql);
    return o;
}

int query(int o, int l, int r, int qr) {
    if (!o) {
        return -1;
    } else if (l == r) {
        return l;
    }
    int mid = (l + r) / 2;
    if (mid + 1 < qr) {
        int ans = query(heap[o].rc, mid + 1, r, qr);
        if (ans != -1) {
            return ans;
        }
    }
    return query(heap[o].lc, l, mid, qr);
}

int query(int o, int qr) {
    return query(o, 0, n - 1, qr);
}

// vector<vector<int>> dss;
//
// int insert(int o, int ind) {
//     dss.push_back(dss[o]);
//     dss.back()[ind] = ind;
//
//     return sz(dss) - 1;
// }
//
// int update_clear(int o, int ql) {
//     dss.push_back(dss[o]);
//     fill(dss.back().begin() + ql, dss.back().end(), -1);
//
//     return sz(dss) - 1;
// }
//
// int query(int o, int qr) {
//     for (int i = qr - 1; i >= 0; i--) {
//         if (dss[o][i] != -1) {
//             return dss[o][i];
//         }
//     }
//
//     return -1;
// }

void solve() {
    cin >> n;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
        a = -a;
    }

    vector<pair<int, int>> comp;
    for (int i = 0; i < n; i++) {
        comp.emplace_back(max(0, arr[i]), i);
    }
    sort(begin(comp), end(comp));

    int ds = 0;
    vector<int> h_ds;

    for (int i = 0; i < n; i++) {
        h_ds.push_back(ds);

        int ds_ind = q_lb(comp, {max(0, arr[i]), i}),
            targ = query(ds, q_lb(comp, {arr[i], -1}));
        ds = targ == -1 ? 0 : h_ds[comp[targ].second];
        ds = update_clear(ds, q_lb(comp, {arr[i], -1}));
        ds = insert(ds, ds_ind);

        // dbg(ds, ds_ind, targ, dss[ds][1], dss[ds][0]);

        int ans = query(ds, q_lb(comp, {1, -1}));

        if (ans == -1) {
            cout << 0 << endl;
        } else {
            cout << -arr[comp[ans].second] << endl;
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}