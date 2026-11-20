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

void init_io() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
}

template <typename T>
T reversed(T arr) {
    reverse(begin(arr), end(arr));
    return arr;
}

template <typename T>
T sorted(T arr) {
    sort(begin(arr), end(arr));
    return arr;
}

mt19937_64 cowng(chrono::steady_clock::now().time_since_epoch().count());

long rint(long l, long r) {
    return cowng() % (r - l + 1) + l;
}

vector<int> tally(const vector<int>& arr) {
    int n = sz(arr);

    vector<int> ans(n);
    for (auto& a : arr) {
        ans[a]++;
    }

    return ans;
}

bool solve12(const vector<int>& arra, const vector<int>& arrb) {
    int n = sz(arra);

    if (*max_element(begin(arra), end(arra)) > 1 ||
        *max_element(begin(arrb), end(arrb)) > 1) {
        return false;
    }

    if (arra == reversed(arrb)) {
        cout << 1 << endl;
        for (int it = 0; it < 2; it++) {
            for (int i = 0; i < n; i++) {
                cout << 1 << " \n"[i == n - 1];
            }
        }
        return true;
    }

    cout << 2 << endl;
    for (auto& cv : {arra, arrb}) {
        for (int i = 0; i < n; i++) {
            cout << cv[i] + 1 << " \n"[i == n - 1];
        }
    }

    return true;
}

bool solve_dist(vector<int> arra, const vector<int>& arrb) {
    int n = sz(arra);
    auto init_arra = arra;
    reverse(begin(arra), end(arra));

    auto is_perm = [&](const vector<int>& arr) -> bool {
        vector<bool> vis(n);

        for (auto& a : arr) {
            if (vis[a]) {
                return false;
            }
            vis[a] = true;
        }

        return true;
    };

    if (!is_perm(arra) || !is_perm(arrb)) {
        return false;
    }

    vector<int> iarra(n);
    for (int i = 0; i < n; i++) {
        iarra[arra[i]] = i;
    }

    vector<int> cf(n), ans_id(n);
    auto cf_l = end(cf);

    for (auto& a : arrb) {
        int cx = iarra[a];

        auto it = --lower_bound(cf_l, end(cf), cx);
        *it = cx;
        dbg(a, end(cf) - it);
        ans_id[a] = int(end(cf) - it);

        if (it == prev(cf_l)) {
            cf_l = it;
        }
    }

    cout << end(cf) - cf_l << endl;
    for (auto& cv : {init_arra, arrb}) {
        for (int i = 0; i < n; i++) {
            cout << ans_id[cv[i]] << " \n"[i == n - 1];
        }
    }

    return true;
}

void solve() {
    int n;
    cin >> n;
    vector<int> arra(n), arrb(n);
    for (auto& a : arra) {
        cin >> a;
        a--;
    }
    for (auto& a : arrb) {
        cin >> a;
        a--;
    }

    if (tally(arra) != tally(arrb)) {
        cout << -1 << endl;
        return;
    }

    bool solved = solve12(arra, arrb) || solve_dist(arra, arrb);
}

int main() {
    init_io();
    solve();
}