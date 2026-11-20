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

constexpr int maxa = 1e7 + 5, maxb = 11;

int c_phi(int x) {
    static int cache[maxa + 1] {};
    if (cache[x]) {
        return cache[x];
    }
    int& ans = cache[x];
    ans = x;
    for (int i = 2; i <= int(1e5); i++) {
        if (x % i != 0) {
            continue;
        }
        ans = ans / i * (i - 1);
        while (x % i == 0) {
            x /= i;
        }
    }
    if (x > 1) {
        ans = ans / x * (x - 1);
    }
    return ans;
}

pair<bool, int> bpow(long base, long exp, int mod) {
    bool of = false, base_of = false;
    long ans = 1;
    while (exp) {
        if (exp & 1) {
            of |= ans * base >= mod || base_of;
            ans = (ans * base) % mod;
        }
        base_of |= base * base >= mod;
        base = (base * base) % mod;
        exp >>= 1;
    }
    return {of, ans};
}

struct SPow {
    static constexpr int logn = 24, low = 18;

    int base, mod, mexp;
    long lift[logn + 1], pcomp[1 << low];

    SPow(int base, int mod) : base(base), mod(mod) {
        {
            long x = 1;
            for (mexp = 0; x < mod; mexp++, x *= base)
                ;
        }
        pcomp[0] = 1;
        for (int i = 1; i < (1 << low); i++) {
            pcomp[i] = (pcomp[i - 1] * base) % mod;
        }
        lift[0] = base;
        for (int i = 1; i <= logn; i++) {
            lift[i] = (lift[i - 1] * lift[i - 1]) % mod;
        }
    }

    pair<bool, int> query(int exp) {
        if (mod == 1) {
            return {true, 0};
        }
        long ans = pcomp[exp & ((1 << low) - 1)];
        for (int i = logn; i >= low; i--) {
            if ((exp >> i) & 1) {
                ans = (ans * lift[i]) % mod;
            }
        }
        pair<bool, int> ans1 {exp >= mexp, ans};
        return ans1;
//        dbg(base, exp, mod, ans1.first, ans1.second);
//        assert(ans1 == bpow(base, exp, mod));
//        return bpow(base, exp, mod);
    }
};

int base;
vector<int> mphis;
vector<SPow> mpows;

pair<bool, int> cpow(int ind, int x, int mind) {
    if (!ind) {
        return {x >= mphis[mind], x % mphis[mind]};
    }
    int nmind = min(sz(mphis) - 1, mind + 1);
    auto [of, nx] = cpow(ind - 1, x, nmind);
    if (of) {
        nx += mphis[nmind];
    }
    dbg(nx);
    auto [cf, ans] = mpows[mind].query(nx);
    return {of || cf, ans};
}

int mod;

struct Node {
    array<int, maxb> v;

    Node operator+(const Node& r) {
        Node ans {};
        for (int i = 0; i < maxb; i++) {
            ans.v[i] = v[i] + r.v[i];
            if (ans.v[i] >= mod) {
                ans.v[i] -= mod;
            }
        }
        return ans;
    }

    Node shift(int ind) const {
        ind = min(ind, maxb);
        Node ans;
        auto ar = copy(v.begin() + ind, v.end(), ans.v.begin());
        fill(ar, ans.v.end(), v.back());
        return ans;
    }

    int operator()(int ind) const {
        return v[min(ind, maxb - 1)];
    }

    static Node c_from(int x) {
        Node ans;
        for (int i = 0; i < maxb; i++) {
            ans.v[i] = cpow(i, x, 0).second;
        }
        return ans;
    }
};

struct ST {
    int n;
    vector<Node> v, arr;
    vector<int> lazy;

    ST(const vector<Node>& arr) : n(sz(arr)), v(4 * n), arr(arr), lazy(4 * n) {
        build(1, 0, n - 1);
    }

    void build(int o, int l, int r) {
        if (l == r) {
            v[o] = arr[l];
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        build(lc, l, mid);
        build(rc, mid + 1, r);
        v[o] = v[lc] + v[rc];
    }

    void update(int o, int l, int r, int ql, int qr, int x) {
        if (ql <= l && r <= qr) {
            v[o] = v[o].shift(x);
            lazy[o] += x;
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ql <= mid) {
            update(lc, l, mid, ql, qr, x);
        }
        if (mid < qr) {
            update(rc, mid + 1, r, ql, qr, x);
        }
        v[o] = (v[lc] + v[rc]).shift(lazy[o]);
    }

    void update(int l, int r, int x) {
        update(1, 0, n - 1, l, r, x);
    }

    int query(int o, int l, int r, int ql, int qr, int clazy) const {
        if (ql <= l && r <= qr) {
            return v[o](clazy);
        }
        clazy += lazy[o];
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ql <= mid) {
            if (mid < qr) {
                return (query(lc, l, mid, ql, qr, clazy) +
                        query(rc, mid + 1, r, ql, qr, clazy)) %
                       mod;
            }
            return query(lc, l, mid, ql, qr, clazy);
        }
        return query(rc, mid + 1, r, ql, qr, clazy);
    }

    int query(int l, int r) const {
        return query(1, 0, n - 1, l, r, 0);
    }
};

void solve() {
    {
        SPow s(2, 1);
        dbg(s.query(0).second);
//        return;
    }
    int n, q;
    IO::read_int(n, q, mod, base);
    int arr[n];
    for (auto& a : arr) {
        IO::read_int(a);
    }
    {
        int x = mod;
        while (true) {
            mphis.push_back(x);
            if (x == 1) {
                break;
            }
            x = c_phi(x);
        }
        for (auto& a : mphis) {
            mpows.emplace_back(base, a);
        }
    }
    vector<Node> arr_v;
    for (auto& a : arr) {
        arr_v.push_back(Node::c_from(a));
    }
    ST arr_st(arr_v);
    while (q--) {
        int t, l, r;
        IO::read_int(t, l, r);
        l--;
        r--;
        if (t == 0) {
            arr_st.update(l, r, 1);
        } else {
            IO::write_int(arr_st.query(l, r), '\n');
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    IO::init();
    solve();
}