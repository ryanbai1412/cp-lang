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
    if (n == 1) {
        return ind;
    }
    assert(n == 26);
    for (int i = 0; i < 26; i++) {
        write(names[i][0] - 'a');
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
    bool aeq = true;
    for (int i = 0; i < m; i++) {
        aeq &= string(senders[i]) == string(recipients[i]);
    }
    if (aeq) {
        fill(outputs_bob, outputs_bob + m, false);
        return m;
    }
    int n = 26;
    bool arr[26][26] {};
    for (int i = 0; i < m; i++) {
        string u(senders[i]), v(recipients[i]);
        assert(sz(u) == 1 && sz(v) == 1);
        arr[u[0] - 'a'][v[0] - 'a'] = true;
    }
    int ind = 0;
    for (int i = 0; i < 26; i++) {
        for (int j = 0; j < 26; j++) {
            outputs_bob[ind++] = arr[i][j];
        }
    }
    return ind;
}

using I = array<int, 16>;

struct Circuit {
    int initn, c0, c1;
    vector<array<int, 3>> arr;

    int f(int op, int u, int v) {
        int ans = sz(arr);
        arr.push_back({op, u, v});
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
        assert(initn);
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
    Circuit c(la + lb);
    if (la == 16) {
        I x = c.read16(0);
        I ans = c.izero();
        for (int i = 0; i < lb; i++) {
            ans = c.add(ans, x);
        }
        c.write(operations, operands);
        copy(begin(ans), end(ans), outputs_circuit[0]);
        return sz(c.arr);
    }
    I arr[26], perm[26];
    for (int i = 0; i < 26; i++) {
        arr[i] = c.read16(16 * i);
    }
    for (int i = 0; i < 26; i++) {
        perm[i] = c.read16(16 * (26 + i));
    }
    int send[26][26];
    for (int i = 0, ind = 0; i < 26; i++) {
        for (int j = 0; j < 26; j++) {
            send[i][j] = la + ind++;
        }
    }
    I res[26];
    for (int i = 0; i < 26; i++) {
        I ci = c.izero();
        for (int j = 0; j < 26; j++) {
            ci = c.add(ci, c.mul_bit(arr[j], send[j][i]));
        }
        res[i] = ci;
    }
    for (int i = 0; i < 26; i++) {
        I cans = c.izero();
        for (int j = 0; j < 26; j++) {
            cans = c.add(cans, c.mul_bit(res[j], c.eq(c.iconst(j), perm[i])));
        }
        copy(begin(cans), end(cans), outputs_circuit[i]);
    }
    c.write(operations, operands);
    return sz(c.arr);
}