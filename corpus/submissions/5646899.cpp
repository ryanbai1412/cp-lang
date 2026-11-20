#ifndef LOCAL
#pragma GCC optimize("Ofast,unroll-loops")
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
ostream& operator<<(ostream& out, const vector<T>& arr) {
    out << "[";
    for (int i = 0; i < sz(arr); i++) {
        if (i) {
            out << " ";
        }
        out << arr[i];
    }
    return out << "]";
}

template <typename T, size_t N>
ostream& operator<<(ostream& out, const array<T, N>& arr) {
    out << "[";
    for (size_t i = 0; i < N; i++) {
        if (i) {
            out << " ";
        }
        out << arr[i];
    }
    return out << "]";
}

struct FastCin {
    char buf[20 << 20], *i = buf;

    FastCin() {
        fread_unlocked(buf, 1, sizeof(buf), stdin);
    }

    FastCin& operator>>(int& x) {
        while (*i < '0') {
            i++;
        }
        x = 0;
        while (*i >= '0') {
            x = x * 10 + (*(i++) ^ '0');
        }
        return *this;
    }
} in;

int p_mod(int x, int mod) {
    x %= mod;
    if (x < 0) {
        x += mod;
    }
    return x;
}

int to_xor(const vector<array<int, 2>>& arr) {
    int ans = 0;
    for (int i = 0; i < sz(arr); i++) {
        for (auto& a : arr[i]) {
            if (a) {
                ans ^= i;
            }
        }
    }
    return ans;
}

int simu_2(vector<array<int, 2>>&& arr, int kv) {
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
            return 0;
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

    int ans = 0, c_shift = p_mod(-kv, n);
    for (int i = 0; i < n; i++) {
        if (arr[i][0]) {
            int cx = i + c_shift;
            if (cx >= n) {
                cx -= n;
            }
            ans ^= cx;
        }
        if (arr[i][1]) {
            ans ^= i;
        }
    }

    return ans;
    // int n_shift[n], c_shift = (-kv) % n;
    // for (int i = 0; i < n; i++) {
    //     n_shift[(i + c_shift + n) % n] = arr[i][0];
    // }
    //
    // for (int i = 0; i < n; i++) {
    //     arr[i][0] = n_shift[i];
    // }
    // return arr;
}

struct DS {
    int n, kv, ans = 0;
    vector<int> arr, b_pair, c_pair, b_xor, c_xor;

    DS(const vector<int>& arr, int kv)
        : n(sz(arr)),
          kv(p_mod(-kv, n)),
          arr(arr),
          b_pair(n, -1),
          c_pair(n, -1),
          b_xor(n),
          c_xor(n) {
        for (int i = 0; i < n; i++) {
            if (arr[i] == -10) {
                continue;
            }
            if (arr[i] <= 0) {
                set_c_pair(i, -2);
            }
            if (arr[i] >= 0) {
                set_b_pair(i, -2);
            }
        }

        vector<int> st;
        for (int it = 0; it < 2; it++) {
            for (int i = n - 1; i >= 0; i--) {
                if (arr[i] == -1 || arr[i] == -10) {
                    if (sz(st)) {
                        dbg(i, st.back());
                        set_b_pair(i, st.back());
                        set_c_pair(st.back(), i);
                        st.pop_back();
                    }
                }
                if (arr[i] == +1 || arr[i] == -10) {
                    st.push_back(i);
                }
            }
        }

        for (int i = 0; i < n; i++) {
            set_b_pair(i, b_pair[i]);
            set_c_pair(i, c_pair[i]);
        }
    }

    void set_b_pair(int ind, int x) {
        ans ^= b_xor[ind];

        if (arr[ind] >= 0 || x != -1) {
            b_xor[ind] = ind;
        } else {
            b_xor[ind] = 0;
        }
        ans ^= b_xor[ind];

        b_pair[ind] = x;
    }

    void set_c_pair(int ind, int x) {
        ans ^= c_xor[ind];

        if ((arr[ind] == -10 || arr[ind] > 0) && x == -1) {
            c_xor[ind] = ind + kv;
            if (c_xor[ind] >= n) {
                c_xor[ind] -= n;
            }
        } else {
            c_xor[ind] = 0;
        }
        ans ^= c_xor[ind];

        c_pair[ind] = x;
    }

    void update(int u) {
        int v = (u + 1) % n;
        arr[u]--;
        arr[v]++;
        if (arr[u] == -1 && arr[v] == 0) {
            if (b_pair[v] == -1) {
                set_b_pair(u, -1);
                set_b_pair(v, -2);
            } else {
                int c = b_pair[v];
                set_b_pair(u, c);
                set_b_pair(v, -2);
                set_c_pair(c, u);
            }
        } else if (arr[u] == -1 && arr[v] == 1) {
            set_b_pair(u, v);
            set_c_pair(v, u);
        } else if (arr[u] == 0 && arr[v] == 0) {
            if (b_pair[v] != -1) {
                int c = b_pair[v];
                if (c_pair[u] == -1) {
                    set_c_pair(c, -1);
                } else {
                    int bu = c_pair[u];
                    set_c_pair(c, bu);
                    set_b_pair(bu, c);
                }
            } else {
                assert(c_pair[u] != -1);
                int b = c_pair[u];
                set_b_pair(b, -1);
            }
            set_b_pair(u, -2);
            set_b_pair(v, -2);
            set_c_pair(u, -2);
            set_c_pair(v, -2);
        } else if (arr[u] == 0 && arr[v] == 1) {
            if (c_pair[u] == -1) {
                set_c_pair(u, -2);
                set_c_pair(v, -1);
            } else {
                int b = c_pair[u];
                set_c_pair(u, -2);
                set_c_pair(v, b);
                set_b_pair(b, v);
            }
        } else {
            dbg(u, v, arr[u], arr[v]);
            assert(false);
        }
    }

    int query_all() const {
        return ans;
    }
};

int simu(const vector<int>& arr, int kv, int ind, int x) {
    int n = sz(arr) / 2 + 1;

    vector<array<int, 2>> sarr(n);

    auto upd = [&](int ind, int x) -> void { sarr[ind >> 1][ind & 1] = x; };

    for (int i = 0; i < 2 * ind; i++) {
        upd(i, arr[i]);
    }
    upd(2 * ind, x);
    for (int i = 2 * ind; i < sz(arr); i++) {
        upd(i + 1, arr[i]);
    }

    return simu_2(std::move(sarr), kv);
}

void simu_0(vector<int> arr, int kv, int* out) {
    int n = sz(arr) / 2 + 1;

    int cmn = int(min_element(begin(arr), end(arr)) - arr.begin());
    if (arr[cmn]) {
        // all 1s except me -> only 0 is at column 0 -> xor 0
        return;
    }

    {
        int cx = simu(arr, kv, 0, 0);
        for (int i = 0; i < cmn / 2; i++) {
            out[i] ^= cx;
        }
    }
    for (int i : {cmn / 2, cmn / 2 + 1}) {
        if (i < n) {
            out[i] ^= simu(arr, kv, i, 0);
        }
    }

    if (cmn / 2 + 2 >= n) {
        return;
    }

    arr.insert(arr.end() - 1, 0);
    swap(arr[0], arr[cmn]);
    arr[0] = 1;

    vector<int> sarr(n);
    for (int i = 0; i < n; i++) {
        sarr[i] = arr[2 * i] + arr[2 * i + 1] - 1;
    }
    if (cmn && !sarr[cmn / 2]) {
        sarr[cmn / 2] = -10;
    }

    dbg(cmn, cmn / 2 + 2, arr, sarr);
    DS ds(sarr, kv);

    for (int i = n - 1; i >= cmn / 2 + 2; i--) {
        out[i] ^= ds.query_all();

        if (i > cmn / 2 + 2 && arr[2 * i - 1]) {
            ds.update(i - 1);
        }
    }
}

void simu_1(vector<int> arr, int kv, int* out) {
    int n = sz(arr) / 2 + 1;

    int cmn = int(min_element(begin(arr), end(arr)) - arr.begin());
    if (arr[cmn]) {
        // all 1s -> xor = 0
        return;
    }

    {
        int cx = simu(arr, kv, 0, 1);
        for (int i = 0; i < cmn / 2; i++) {
            out[i] ^= cx;
        }
    }
    for (int i : {cmn / 2, cmn / 2 + 1}) {
        if (i < n) {
            out[i] ^= simu(arr, kv, i, 1);
        }
    }

    if (cmn / 2 + 2 >= n) {
        return;
    }

    arr.insert(arr.begin() + 2 * (cmn / 2 + 2), 1);
    swap(arr[0], arr[cmn]);
    arr[0] = 1;

    vector<int> sarr(n);
    for (int i = 0; i < n; i++) {
        sarr[i] = arr[2 * i] + arr[2 * i + 1] - 1;
    }
    if (cmn && !sarr[cmn / 2]) {
        sarr[cmn / 2] = -10;
    }

    DS ds(sarr, kv);

    for (int i = cmn / 2 + 2; i < n; i++) {
        out[i] ^= ds.query_all();

        if (i + 1 < n && !arr[2 * i + 2]) {
            ds.update(i);
        }
    }
}

void solve() {
    int n, kv, mx;
    in >> n >> kv >> mx;
    mx--;
    vector<int> arr(2 * n - 1);
    for (auto& a : arr) {
        in >> a;
        a--;
        a = a >= mx;
    }

    int s_ans[n] {};
    simu_0(arr, kv, s_ans);
    simu_1(arr, kv, s_ans);
    // auto bc0 = simu_0_bf(arr, kv);
    dbg(arr);
    // dbg(c0);
    // dbg(bc0);
    // assert(c0 == bc0);

    pair<int, int> ans {int(1e9), -1};
    for (int i = 0; i < n; i++) {
        ans = min(ans, {s_ans[i], -i});
    }
    cout << -ans.second + 1 << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}