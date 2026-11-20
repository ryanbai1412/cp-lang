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

template <typename T>
T reversed(T arr) {
    reverse(begin(arr), end(arr));
    return arr;
}

int solve(const vector<int>& arr, const vector<int>& par, const vector<int>& ty) {
    int n = sz(arr);

    vector<int> w0(n), w1 = arr;

    for (int u = n - 1; u >= 1; u--) {
        int p = par[u];

        if (ty[u] == 0) {
            w0[p] += max(w0[u], w1[u]);
            w1[p] += w0[u];
        } else if (ty[u] == 1) {
            w1[p] = max(w1[p] + max(w0[u], w1[u]), w0[p] + w1[u]);
            w0[p] += w0[u];
        } else {
            w1[p] = max(w1[p] + w0[u], w0[p] + w1[u]);
            w0[p] += w0[u];
        }
    }

    return max(w0[0], w1[0]);
}

int findSample(int n, int confidence[], int host[], int protocol[]) {
    return solve(vector(confidence, confidence + n), vector(host, host + n),
                 vector(protocol, protocol + n));
}