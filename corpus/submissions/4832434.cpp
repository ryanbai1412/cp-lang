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

struct PSA {
    vector<int> psum;

    PSA(const vector<int>& arr) {
        psum.push_back(0);
        for (auto& a : arr) {
            psum.push_back(psum.back() + a);
        }
    }

    int query(int l, int r) const {
        if (l >= r) {
            return 0;
        }
        return psum[r] - psum[l];
    }
};

int coord_comp(vector<int*> arr) {
    sort(begin(arr), end(arr), [&](const int* a, const int* b) -> bool {
        return *a < *b;
    });
    int j = 0, prv;
    for (int i = 0; i < sz(arr); i++) {
        j += i && prv != *arr[i];
        prv = *arr[i];
        *arr[i] = j;
    }
    return j + 1;
}

void solve() {
    int n, m;
    cin >> n >> m;
    int conds[m][3];
    for (auto& [l, r, x] : conds) {
        cin >> l >> r >> x;
        l--;
        r--;
    }
    {
        vector<int*> comp;
        for (auto& a : conds) {
            comp.push_back(&a[2]);
        }
        m = coord_comp(comp);
    }
    int arr[n] {};
    bool eqs[n] {};
    for (auto& [l, r, x] : conds) {
        fill(arr + l, arr + r + 1, x);
        eqs[l] = true;
    }
    long ans = 0;
    for (int i = 0; i < n; i++) {
        if (eqs[i]) {
            continue;
        }
        vector<int> lv(m), rv(m);
        for (int j = 0; j < i; j++) {
            lv[arr[j]]++;
        }
        for (int j = i + 1; j < n; j++) {
            if (eqs[j]) {
                rv[arr[j]]++;
            }
        }
        PSA lp(lv), rp(rv);
        int cans = 1e9;
        for (int j = arr[i]; j < m; j++) {
            cans = min(cans, lp.query(j + 1, m) + rp.query(0, j));
        }
        ans += cans;
    }
    for (int i = 0; i < n; i++) {
        for (int j = i + 1; j < n; j++) {
            if (eqs[i] && eqs[j] && arr[i] > arr[j]) {
                ans++;
            }
        }
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}