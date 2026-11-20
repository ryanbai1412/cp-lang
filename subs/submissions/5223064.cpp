#pragma GCC optimize("Ofast")
#pragma GCC target("avx2")

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

using u64 = uint64_t;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

namespace IO {
const int BUFFER_SIZE = 1 << 16;

char input_buffer[BUFFER_SIZE];
size_t input_pos = 0, input_len = 0;

char output_buffer[BUFFER_SIZE];
int output_pos = 0;

char number_buffer[100];
uint8_t lookup[100];

void _update_input_buffer() {
    input_len = fread(input_buffer, sizeof(char), BUFFER_SIZE, stdin);
    input_pos = 0;

    if (input_len == 0)
        input_buffer[0] = EOF;
}

inline char next_char(bool advance = true) {
    if (input_pos >= input_len)
        _update_input_buffer();

    return input_buffer[advance ? input_pos++ : input_pos];
}

template <typename T>
inline void read_int(T& number) {
    bool negative = false;
    number = 0;

    while (!isdigit(next_char(false)))
        if (next_char() == '-')
            negative = true;

    do {
        number = 10 * number + (next_char() - '0');
    } while (isdigit(next_char(false)));

    if (negative)
        number = -number;
}

template <typename T, typename... Args>
inline void read_int(T& number, Args&... args) {
    read_int(number);
    read_int(args...);
}

void _flush_output() {
    fwrite(output_buffer, sizeof(char), output_pos, stdout);
    output_pos = 0;
}

inline void write_char(char c) {
    if (output_pos == BUFFER_SIZE)
        _flush_output();

    output_buffer[output_pos++] = c;
}

template <typename T>
inline void write_int(T number, char after = '\0') {
    if (number < 0) {
        write_char('-');
        number = -number;
    }

    int length = 0;

    while (number >= 10) {
        uint8_t lookup_value = lookup[number % 100];
        number /= 100;
        number_buffer[length++] = char((lookup_value & 15) + '0');
        number_buffer[length++] = char((lookup_value >> 4) + '0');
    }

    if (number != 0 || length == 0)
        write_char(char(number + '0'));

    for (int i = length - 1; i >= 0; i--)
        write_char(number_buffer[i]);

    if (after)
        write_char(after);
}

void init() {
    // Make sure _flush_output() is called at the end of the program.
    bool exit_success = atexit(_flush_output) == 0;
    assert(exit_success);

    for (int i = 0; i < 100; i++)
        lookup[i] = uint8_t((i / 10 << 4) + i % 10);
}
}  // namespace IO

template <typename T, size_t N, size_t M>
using Mat = array<array<T, M>, N>;

template <typename T, size_t A, size_t B, size_t C>
Mat<T, A, C> operator*(const Mat<T, A, B>& a, const Mat<T, B, C>& b) {
    Mat<T, A, C> ans {};
    for (size_t i = 0; i < A; i++) {
        for (size_t j = 0; j < B; j++) {
            for (size_t k = 0; k < C; k++) {
                ans[i][k] += a[i][j] * b[j][k];
            }
        }
    }
    return ans;
}

struct Node {
    array<u64, 5> v;

    Node& operator+=(const Node& n) {
        for (int i = 0; i < 5; i++) {
            v[i] += n.v[i];
        }
        return *this;
    }

    friend Node operator+(Node a, const Node& b) {
        return a += b;
    }

    static Node c_def() {
        return {1, 0, 0, 0, 0};
    }
};

int offset(int i) {
    i = 5 - i;
    return 15 - i * (i + 1) / 2;
}

struct Lazy {
    array<u64, 15> v;

    Lazy& operator+=(const Lazy& l) {
        array<u64, 15> ans = v;
        for (int i = 0; i < 15; i++) {
            ans[i] += l.v[i];
        }
#pragma GCC unroll 5
        for (int i = 0; i < 5; i++) {
            int oi = offset(i);
#pragma GCC unroll 5
            for (int j = i + 1; j < 5; j++) {
                int oj = offset(j);
#pragma GCC unroll 5
                for (int k = j + 1; k < 5; k++) {
                    ans[oi + k - i] += v[oi + j - i] * l.v[oj + k - j];
                }
            }
        }
#pragma GCC unroll 5
        for (int i = 0; i < 5; i++) {
            ans[offset(i)] = 1;
        }
        v = ans;
        return *this;
    }

    friend Lazy operator+(Lazy a, const Lazy& b) {
        return a += b;
    }

    void apply_inplace(Node& n) const {
        array<u64, 5> ans = n.v;
#pragma GCC unroll 5
        for (int j = 0; j < 5; j++) {
            int oj = offset(j);
#pragma GCC unroll 5
            for (int k = j + 1; k < 5; k++) {
                ans[k] += n.v[j] * v[oj + k - j];
            }
        }
        n.v = ans;
    }

    Node apply(Node n) const {
        apply_inplace(n);
        return n;
    }

    //    static Lazy c_def() {
    //        return {{{{1, 0, 0, 0, 0},
    //                  {0, 1, 0, 0, 0},
    //                  {0, 0, 1, 0, 0},
    //                  {0, 0, 0, 1, 0},
    //                  {0, 0, 0, 0, 1}}}};
    //    }
    //
    //    static Lazy c_adda(u64 x) {
    //        return {{{{1, x, 0, 0, 0},
    //                  {0, 1, 0, 0, 0},
    //                  {0, 0, 1, x, 0},
    //                  {0, 0, 0, 1, 0},
    //                  {0, 0, 0, 0, 1}}}};
    //    }
    //
    //    static Lazy c_addb(u64 x) {
    //        return {{{{1, 0, x, 0, 0},
    //                  {0, 1, 0, x, 0},
    //                  {0, 0, 1, 0, 0},
    //                  {0, 0, 0, 1, 0},
    //                  {0, 0, 0, 0, 1}}}};
    //    }
    //
    //    static Lazy c_h_add() {
    //        return {{{{1, 0, 0, 0, 0},
    //                  {0, 1, 0, 0, 0},
    //                  {0, 0, 1, 0, 0},
    //                  {0, 0, 0, 1, 1},
    //                  {0, 0, 0, 0, 1}}}};
    //    }

    static Lazy c_def() {
        return {{{1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 0, 1}}};
    }

    static Lazy c_adda(u64 x) {
        return {{{1, x, 0, 0, 0, 1, 0, 0, 0, 1, x, 0, 1, 0, 1}}};
    }

    static Lazy c_addb(u64 x) {
        return {{{1, 0, x, 0, 0, 1, 0, x, 0, 1, 0, 0, 1, 0, 1}}};
    }

    static Lazy c_h_add() {
        return {{{1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 1}}};
    }

    static Lazy c_add(bool b, u64 x) {
        return b ? c_addb(x) : c_adda(x);
    }
};

struct ST {
    int n;
    vector<Node> v;
    vector<Lazy> lazy;

    ST(int n) : n(n), v(4 * n), lazy(4 * n, Lazy::c_def()) {}

    void build(int o, int l, int r) {
        if (l == r) {
            v[o] = Node::c_def();
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        build(lc, l, mid);
        build(rc, mid + 1, r);
        v[o] = v[lc] + v[rc];
    }

    void maintain(int o, int l, int r) {
        if (l == r) {
            v[o] = Node::c_def();
        } else {
            v[o] = v[o * 2] + v[o * 2 + 1];
        }
        lazy[o].apply_inplace(v[o]);
    }

    void update(int o, int l, int r, int ql, int qr, const Lazy& x) {
        if (ql <= l && r <= qr) {
            lazy[o] += x;
            maintain(o, l, r);
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        lazy[lc] += lazy[o];
        lazy[rc] += lazy[o];
        lazy[o] = Lazy::c_def();
        if (ql <= mid) {
            update(lc, l, mid, ql, qr, x);
        } else {
            maintain(lc, l, mid);
        }
        if (mid < qr) {
            update(rc, mid + 1, r, ql, qr, x);
        } else {
            maintain(rc, mid + 1, r);
        }
        maintain(o, l, r);
    }

    void update(int l, int r, const Lazy& x) {
        update(1, 0, n - 1, l, r, x);
    }

    Node query(int o, int l, int r, int ql, int qr, Lazy clazy) const {
        if (ql <= l && r <= qr) {
            return clazy.apply(v[o]);
        }
        clazy = lazy[o] + clazy;
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ql <= mid) {
            if (mid < qr) {
                return query(lc, l, mid, ql, qr, clazy) +
                       query(rc, mid + 1, r, ql, qr, clazy);
            }
            return query(lc, l, mid, ql, qr, clazy);
        }
        return query(rc, mid + 1, r, ql, qr, clazy);
    }

    Node query(int l, int r) const {
        return query(1, 0, n - 1, l, r, Lazy::c_def());
    }
};

struct DS1 {
    vector<pair<int, int>> st;

    DS1(int n) : st {{n, 1e9}} {}

    template <typename Cb>
    void update(int ind, int x, const Cb& cb) {
        while (st.back().second < x) {
            cb(st.back().first, st[sz(st) - 2].first - 1, st.back().second);
            st.pop_back();
        }
        cb(ind, ind, 0);
        st.emplace_back(ind, x);
    }
};

void solve() {
    //    {
    //        Lazy lazy = Lazy::c_def();
    //        for (int iter = 0; iter < 30; iter++) {
    //            int cur = rand() % 3;
    //            if (cur == 0) {
    //                lazy += Lazy::c_adda(1);
    //            } else if (cur == 1) {
    //                lazy += Lazy::c_addb(2);
    //            } else {
    //                lazy += Lazy::c_h_add();
    //            }
    //            for (int i = 0; i < 5; i++) {
    //                for (int j = 0; j < 5; j++) {
    //                    cerr << lazy.v[i][j] << " \n"[j == 4];
    //                }
    //            }
    //            cerr << "----" << endl;
    //        }
    //        return;
    //    }
    int _id, n;
    IO::read_int(_id, n);
    vector<int> arr[2];
    for (auto& a : arr) {
        a.resize(n);
        for (auto& b : a) {
            IO::read_int(b);
        }
    }
    int q;
    IO::read_int(q);
    u64 qans[q];
    vector<pair<int, int>> queries[n];
    for (int i = 0; i < q; i++) {
        int l, r;
        IO::read_int(l, r);
        l--;
        r--;
        queries[l].emplace_back(r, i);
    }
    ST st(n);
    DS1 ds[2] {DS1(n), DS1(n)};
    for (int i = n - 1; i >= 0; i--) {
        for (int j = 0; j < 2; j++) {
            ds[j].update(i, arr[j][i], [&](int l, int r, int x) -> void {
                st.update(l, r, Lazy::c_add(j, arr[j][i] - x));
            });
        }
        st.update(0, n - 1, Lazy::c_h_add());
        for (auto& [r, qi] : queries[i]) {
            qans[qi] = st.query(i, r).v[4];
        }
    }
    for (auto& a : qans) {
        IO::write_int(a, '\n');
    }
}

int main() {
    //    freopen("match.in", "r", stdin);
    //    freopen("match.out", "w", stdout);
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    IO::init();
    solve();
}