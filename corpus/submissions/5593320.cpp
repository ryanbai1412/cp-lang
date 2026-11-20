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

using u32 = uint32_t;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

// you may find the definitions useful
const int OP_ZERO = 0;     // f(OP_ZERO,    x0, x1) = 0
const int OP_NOR = 1;      // f(OP_NOR,     x0, x1) = !(x0 || x1)
const int OP_GREATER = 2;  // f(OP_GREATER, x0, x1) = (x0 > x1)
const int OP_NOT_X1 = 3;   // f(OP_NOT_X1,  x0, x1) = !x1
const int OP_LESS = 4;     // f(OP_LESS,    x0, x1) = (x0 < x1)
const int OP_NOT_X0 = 5;   // f(OP_NOT_X0,  x0, x1) = !x0
const int OP_XOR = 6;      // f(OP_XOR,     x0, x1) = (x0 ^ x1)
const int OP_NAND = 7;     // f(OP_NAND,    x0, x1) = !(x0 && x1)
const int OP_AND = 8;      // f(OP_AND,     x0, x1) = (x0 && x1)
const int OP_EQUAL = 9;    // f(OP_EQUAL,   x0, x1) = (x0 == x1)
const int OP_X0 = 10;      // f(OP_X0,      x0, x1) = x0
const int OP_GEQ = 11;     // f(OP_GEQ,     x0, x1) = (x0 >= x1)
const int OP_X1 = 12;      // f(OP_X1,      x0, x1) = x1
const int OP_LEQ = 13;     // f(OP_LEQ,     x0, x1) = (x0 <= x1)
const int OP_OR = 14;      // f(OP_OR,      x0, x1) = (x0 || x1)
const int OP_ONE = 15;     // f(OP_ONE,     x0, x1) = 1

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

u32 conv_name(string s) {
    assert(sz(s) <= 4);
    s.resize(4, '.');
    return (u32(s[0]) << 24) | (u32(s[1]) << 16) | (u32(s[2]) << 8) | u32(s[3]);
}

// Alice
int  // returns la
alice(
    /*  in */ const int n,
    /*  in */ const char names[][5],
    /*  in */ const unsigned short numbers[],
    /* out */ bool outputs_alice[]) {
    int ind = 0;
    auto write = [&](int x) -> void {
        for (int j = 0; j < 16; j++) {
            outputs_alice[ind++] = on(x, j);
        }
    };
    for (int i = 0; i < n; i++) {
        write(numbers[i]);
    }
    for (int i = 0; i < n; i++) {
        string cn(names[i]);
        cn.resize(4, '.');
        write(conv_name(cn) >> 16);
        write(conv_name(cn) & ((1 << 16) - 1));
    }
    return ind;
}

// Bob
int  // returns lb
bob(
    /*  in */ const int m,
    /*  in */ const char senders[][5],
    /*  in */ const char recipients[][5],
    /* out */ bool outputs_bob[]) {
    vector<pair<string, string>> edges;
    for (int i = 0; i < m; i++) {
        edges.emplace_back(senders[i], recipients[i]);
    }
    vector<string> names;
    for (auto& [u, v] : edges) {
        names.push_back(u);
        names.push_back(v);
    }
    sort(begin(names), end(names));
    names.erase(unique(begin(names), end(names)), names.end());
    if (sz(names) == 1) {
        fill(outputs_bob, outputs_bob + m, false);
        return m;
    }
    int ind = 0;
    auto write16 = [&](int x) -> void {
        for (int j = 0; j < 16; j++) {
            outputs_bob[ind++] = on(x, j);
        }
    };
    for (auto& a : names) {
        write16(conv_name(a) >> 16);
        write16(conv_name(a) & ((1 << 16) - 1));
    }
    int n = sz(names);
    bool arr[n][n] {};
    auto conv = [&](const string& s) -> int {
        return int(lower_bound(begin(names), end(names), s) - names.begin());
    };
    for (int i = 0; i < m; i++) {
        string u(senders[i]), v(recipients[i]);
        arr[conv(u)][conv(v)] = true;
    }
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) {
            outputs_bob[ind++] = arr[i][j];
        }
    }
    return ind;
}

using I = array<int, 16>;

struct Circuit {
    int initn, c0, c1, sptr = 0;
    vector<array<int, 3>> arr;

    I read16() {
        I ans;
        iota(begin(ans), end(ans), sptr);
        sptr += 16;
        assert(sptr <= initn);
        return ans;
    }

    int readbit() {
        return sptr++;
    }

    int f(int op, int u, int v) {
        int ans = sz(arr);
        arr.push_back({op, u, v});
        assert(sptr <= initn);
        return ans;
    }

    int qxor(int u, int v) {
        return f(OP_XOR, u, v);
    }

    int qor(int u, int v) {
        return f(OP_OR, u, v);
    }

    int qand(int u, int v) {
        return f(OP_AND, u, v);
    }

    Circuit(int initn) : initn(initn), arr(initn, {-1, -1, -1}) {
        // assert(initn);
        c0 = f(OP_ZERO, 0, 0);
        c1 = f(OP_ONE, 0, 0);
    }

    I izero() const {
        I ans;
        fill(begin(ans), end(ans), c0);
        return ans;
    }

    I iconst(int x) const {
        I ans;
        for (int i = 0; i < 16; i++) {
            if (on(x, i)) {
                ans[i] = c1;
            } else {
                ans[i] = c0;
            }
        }
        return ans;
    }

    int eq(I a, I b) {
        int ans = c1;
        for (int i = 0; i < 16; i++) {
            ans = qand(ans, f(OP_EQUAL, a[i], b[i]));
        }
        return ans;
    }

    int eq(pair<I, I> a, pair<I, I> b) {
        return qand(eq(a.first, b.first), eq(a.second, b.second));
    }

    I read16(int ind) const {
        I ans;
        iota(begin(ans), end(ans), ind);
        return ans;
    }

    I mul_bit(I x, int bit) {
        I ans;
        for (int i = 0; i < 16; i++) {
            ans[i] = qand(x[i], bit);
        }
        return ans;
    }

    I add(I a, I b) {
        int carry = c0;
        I ans;
        for (int i = 0; i < 16; i++) {
            ans[i] = qxor(qxor(a[i], b[i]), carry);
            carry = qor(qor(qand(a[i], b[i]), qand(a[i], carry)),
                        qand(b[i], carry));
        }
        return ans;
    }

    void write(int operations[], int operands[][2]) {
        for (int i = initn; i < sz(arr); i++) {
            assert(arr[i][0] != -1);
            operations[i] = arr[i][0];
            operands[i][0] = arr[i][1];
            operands[i][1] = arr[i][2];
        }
    }
};

// Circuit
int  // returns l
circuit(
    /*  in */ const int la,
    /*  in */ const int lb,
    /* out */ int operations[],
    /* out */ int operands[][2],
    /* out */ int outputs_circuit[][16]) {
    if (!la && !lb) {
        return 0;
    }
    Circuit c(la + lb);
    int n = la / 48;
    I arr[n];
    pair<I, I> alice_names[n], bob_names[n];
    for (int i = 0; i < n; i++) {
        arr[i] = c.read16();
    }
    for (int i = 0; i < n; i++) {
        alice_names[i] = {c.read16(), c.read16()};
    }
    assert(c.sptr == la);
    if (n == 1) {
        I ans = c.izero();
        for (int i = 0; i < lb; i++) {
            ans = c.add(ans, arr[0]);
        }
        copy(begin(ans), end(ans), outputs_circuit[0]);
        c.write(operations, operands);
        return sz(c.arr);
    }
    for (int i = 0; i < n; i++) {
        bob_names[i] = {c.read16(), c.read16()};
    }
    int send[n][n];
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) {
            send[i][j] = c.readbit();
            dbg(i, j, send[i][j]);
        }
    }
    I res[n];
    for (auto& a : res) {
        a = c.izero();
    }
    for (int i = 0; i < n; i++) {
        I cval = c.izero();
        for (int j = 0; j < n; j++) {
            cval = c.add(cval,
                         c.mul_bit(arr[j], c.eq(alice_names[j], bob_names[i])));
        }
        for (int j = 0; j < n; j++) {
            res[j] = c.add(res[j], c.mul_bit(cval, send[i][j]));
        }
    }
    for (int i = 0; i < n; i++) {
        I cans = c.izero();
        for (int j = 0; j < n; j++) {
            cans = c.add(cans,
                         c.mul_bit(res[j], c.eq(alice_names[i], bob_names[j])));
        }
        copy(begin(cans), end(cans), outputs_circuit[i]);
    }
    c.write(operations, operands);
    return sz(c.arr);
}