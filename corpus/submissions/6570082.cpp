#pragma GCC optimize("Ofast")
#pragma GCC target("avx2")

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

template <typename A, typename B>
ostream& operator<<(ostream& out, const pair<A, B>& p) {
    return out << "(" << p.first << ", " << p.second << ")";
}

template <typename Cb>
struct CmpByKey {
    Cb cb;

    CmpByKey(Cb cb) : cb(cb) {}

    template <typename T>
    bool operator()(const T& a, const T& b) const {
        return cb(a) < cb(b);
    }
};

namespace st1 {

struct Node {
    int lc, rc, v;
} heap[(200 << 20) / sizeof(Node)];
int n, hind = 1;

void update(int& o, int l, int r, int ind, int x) {
    heap[hind] = heap[o];
    o = hind++;
    if (l == r) {
        heap[o].v += x;
        return;
    }

    int mid = (l + r) / 2;
    if (ind <= mid) {
        update(heap[o].lc, l, mid, ind, x);
    } else {
        update(heap[o].rc, mid + 1, r, ind, x);
    }

    heap[o].v = heap[heap[o].lc].v + heap[heap[o].rc].v;
}

void update(int& root, int ind, int x) {
    update(root, 0, n - 1, ind, x);
}

int query_pref(int o, int qr) {
    int ans = 0, l = 0, r = n - 1;

    while (true) {
        if (!o) {
            return ans;
        } else if (r <= qr) {
            return ans + heap[o].v;
        }

        int mid = (l + r) / 2;
        if (qr <= mid) {
            o = heap[o].lc;
            r = mid;
        } else {
            ans += heap[heap[o].lc].v;
            l = mid + 1;
            o = heap[o].rc;
        }
    }
}

}  // namespace st1

struct Solver {
    int n;
    vector<pair<int, int>> arr;
    vector<int> roots;

    Solver() {}
    Solver(const vector<pair<int, int>>& _arr)
    : n(sz(_arr)), arr(_arr), roots(n + 1) {
        sort(begin(arr), end(arr), CmpByKey([&](const auto& a) -> int {
            return -a.second;
        }));
        st1::n = n + 5;

        // assert(n <= 1e5);
        int root = 0;
        for (int i = n, j = 0; i >= 0; i--) {
            for (; j < n && arr[j].second == i; j++) {
                st1::update(root, arr[j].first, +1);
            }

            roots[i] = root;
        }

        sort(begin(arr), end(arr), CmpByKey([&](const auto& a) -> int {
            return a.first;
        }));
    }

    bool query2(vector<pair<int, int>> carr) {
        int m = sz(carr);

        int va[m + 1][m + 1] {};
        {
            int comp1[n + 10], comp2[n + 10];
            for (int i = 0; i < m; i++) {
                int cr = i + 1 < m ? carr[i + 1].first : n + 5;
                fill(comp1 + carr[i].first + 1, comp1 + cr + 1, i + 1);
                fill(comp2 + max(0, carr[i].first), comp2 + cr, i);
            }

            for (auto& [l, r] : arr) {
                va[comp1[l]][comp2[r]]++;
            }
        }

        for (int i = 1; i < m; i++) {
            for (int j = m - 1; j >= 0; j--) {
                va[i][j] =
                    va[i - 1][j] + va[i][j + 1] - va[i - 1][j + 1] + va[i][j];
            }
        }
        int va2[m];
        for (int i = 0; i < m; i++) {
            va2[i] = va[i][i];
        }

        int dp[m] {};

        for (int i = m - 1; i >= 0; i--) {
            for (int j = i + 1; j < m; j++) {
                dp[i] = min(dp[i], va2[j] - va[i][j] + dp[j] - carr[j].second);
            }
        }

        return dp[0] >= 0;
    }

    vector<pair<int, int>> carr;
    bool query(vector<int> qarr) {
        if (accumulate(begin(qarr), end(qarr), long(0)) > n) {
            return false;
        }

        carr.clear();
        sort(begin(qarr), end(qarr));
        for (int i = 0; i < sz(qarr);) {
            int cl = i;
            for (; i < sz(qarr) && qarr[cl] == qarr[i]; i++)
                ;
            carr.emplace_back(qarr[cl], (i - cl) * qarr[cl]);
        }
        carr.insert(begin(carr), pair {-1, 0});

        if (sz(carr) > 500) {
            return query2(carr);
        }
        int m = sz(carr);

        int dp[m] {};
        for (int j = m - 1; j >= 1; j--) {
            int qr = carr[j].first, cv = st1::query_pref(roots[qr], qr);

            for (int i = 0; i < j; i++) {
                dp[i] =
                    min(dp[i], cv - st1::query_pref(roots[qr], carr[i].first) +
                                   dp[j] - carr[j].second);
            }
        }

        return dp[0] >= 0;
    }
} solver;

void init(int n, int arra[], int arrb[]) {
    vector<pair<int, int>> cv(n);

    for (int i = 0; i < n; i++) {
        cv[i] = {arra[i], arrb[i]};
    }

    solver = Solver(cv);
}

int can(int m, int arr[]) {
    return solver.query(vector(arr, arr + m));
}

namespace {
static char _buffer[1 << 16];
static int _currentChar = 0;
static int _charsNumber = 0;
static FILE *_inputFile, *_outputFile;

static inline int _read() {
    if (_charsNumber < 0) {
        exit(1);
    }
    if (!_charsNumber || _currentChar == _charsNumber) {
        _charsNumber = (int)fread(_buffer, sizeof(_buffer[0]), sizeof(_buffer),
                                  _inputFile);
        _currentChar = 0;
    }
    if (_charsNumber <= 0) {
        return -1;
    }
    return _buffer[_currentChar++];
}

static inline int _readInt() {
    int c, x, s;
    c = _read();
    while (c <= 32)
        c = _read();
    x = 0;
    s = 1;
    if (c == '-') {
        s = -1;
        c = _read();
    }
    while (c > 32) {
        x *= 10;
        x += c - '0';
        c = _read();
    }
    if (s < 0)
        x = -x;
    return x;
}
}  // namespace

int main() {
    _inputFile = stdin;
    _outputFile = stdout;
    int N;
    N = _readInt();
    int* A = (int*)malloc(sizeof(int) * (unsigned int)N);
    int* B = (int*)malloc(sizeof(int) * (unsigned int)N);
    for (int i = 0; i < N; ++i) {
        A[i] = _readInt();
        B[i] = _readInt();
    }
    init(N, A, B);
    int Q;
    Q = _readInt();
    for (int i = 0; i < Q; ++i) {
        int M;
        M = _readInt();
        int* K = (int*)malloc(sizeof(int) * (unsigned int)M);
        for (int j = 0; j < M; ++j) {
            K[j] = _readInt();
        }
        fprintf(_outputFile, "%d\n", can(M, K));
        free(K);
    }
    free(A);
    free(B);
    return 0;
}