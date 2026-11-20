// cf bits/extc++.h nonsense
#ifdef ONLINE_JUDGE
#define _EXT_CODECVT_SPECIALIZATIONS_H 1
#define _EXT_ENC_FILEBUF_H 1
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
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__)
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

using ll = long long;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

inline void init_io() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
}

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

template <typename T>
struct PSA {
    int n;
    vector<T> psum;

    template <typename U>
    PSA(const vector<U>& arr) : n(sz(arr)), psum(n + 1) {
        for (int i = 0; i < n; i++) {
            psum[i + 1] = psum[i] + arr[i];
        }
    }

    T query(int l, int r) const {
        return psum[r] - psum[l];
    }
};

template <typename T>
bool on(T mask, int bit) {
    return (mask >> bit) & 1;
}

template <typename T>
vector<T> iota(int n, T x) {
    vector<T> arr(n);
    iota(begin(arr), end(arr), x);
    return arr;
}

void solve() {
    int n, q;
    cin >> n >> q;

    vector<int> arr(n);
    for (auto& a : arr) {
        cin >> a;
    }

    while (q--) {
        string s;
        cin >> s;

        if (s == "MAX-SUM") {
            int ans = -1e9, csum = 0;
            for (auto& a : arr) {
                csum += a;
                ans = max(ans, csum);
                csum = max(csum, 0);
            }
            cout << ans << endl;
            continue;
        }

        if (s == "INSERT") {
            int pos, len;
            cin >> pos >> len;

            vector<int> cvals(len);
            for (auto& a : cvals) {
                cin >> a;
            }

            arr.insert(begin(arr) + pos, begin(cvals), end(cvals));
            continue;
        }

        int ind, len;
        cin >> ind >> len;
        ind--;

        if (s == "DELETE") {
            arr.erase(begin(arr) + ind, begin(arr) + ind + len);
        } else if (s == "MAKE-SAME") {
            int x;
            cin >> x;
            for (int i = 0; i < len; i++) {
                arr[ind + i] = x;
            }
        } else if (s == "REVERSE") {
            reverse(begin(arr) + ind, begin(arr) + ind + len);
        } else if (s == "GET-SUM") {
            cout << accumulate(begin(arr) + ind, begin(arr) + ind + len, 0)
                 << endl;
        }
    }
}

int main() {
    init_io();
    solve();
}