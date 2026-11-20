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

vector<array<int, 2>> simu_2(int kv, vector<array<int, 2>> arr) {
    int n = sz(arr);
    assert(kv >= 2 * n);

    for (auto& a : arr) {
        if (a[0]) {
            swap(a[0], a[1]);
        }
    }

    if (!arr[0][0]) {
        swap(arr[0][0], arr[0][1]);
    }
    if (arr[0][1]) {
        for (int i = 1; i < n; i++) {
            if (!arr[i][0]) {
                swap(arr[0][1], arr[i][0]);
                break;
            }
        }
        if (arr[0][1]) {
            // everything is 1
            return arr;
        }
    }

    vector<int> st;
    for (int it = 0; it < 2; it++) {
        for (int i = n - 1; i >= 0; i--) {
            if (i && !arr[i][1] && sz(st)) {
                int u = st.back();
                st.pop_back();
                swap(arr[i][1], arr[u][0]);
            }
            if (arr[i][0]) {
                st.push_back(i);
            }
        }
    }

    int n_shift[n], c_shift = (-kv) % n;
    for (int i = 0; i < n; i++) {
        n_shift[(i + c_shift + n) % n] = arr[i][0];
    }

    for (int i = 0; i < n; i++) {
        arr[i][0] = n_shift[i];
    }
    return arr;
}

int solve_pos(int kv, int x, vector<array<int, 2>> arr) {
    int n = sz(arr);

    auto simu_bin = [&](int x) -> vector<array<int, 2>> {
        auto ans = arr;
        for (auto& a : ans) {
            for (auto& b : a) {
                b = b >= x;
            }
        }
        return simu_2(kv, ans);
    };

    auto c0 = simu_bin(x + 1), c1 = simu_bin(x);
    for (int i = 0; i < n; i++) {
        sort(begin(c0[i]), end(c0[i]));
        sort(begin(c1[i]), end(c1[i]));
        if (c0[i] != c1[i]) {
            return i;
        }
    }
    assert(false);
}

void solve() {
    int n, kv, mx;
    cin >> n >> kv >> mx;
    mx--;
    int arr[2 * n - 1];
    for (auto& a : arr) {
        cin >> a;
        a--;
    }
    pair<int, int> ans {int(1e9), 1};
    for (int i = 0; i < n; i++) {
        vector<int> carr(arr, arr + 2 * n - 1);
        carr.insert(carr.begin() + 2 * i, mx);
        vector<array<int, 2>> cx(n);
        for (int j = 0; j < n; j++) {
            for (int k = 0; k < 2; k++) {
                cx[j][k] = carr[j * 2 + k];
            }
        }
        ans = min(ans, {solve_pos(kv, mx, cx), -i});
    }
    cout << -ans.second + 1 << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}