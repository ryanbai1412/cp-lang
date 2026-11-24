// noi16p6 (output-only): print a computer for the task in the input. Correctness comes first, so
// the comparison, max and multiplication nodes are used freely.
#include <bits/stdc++.h>
using namespace std;

vector<string> out;
int node(const string& s) { out.push_back(s); return out.size(); }
string N(int x) { return to_string(x); }
int I() { return node("I"); }
void O(int x) { node("O " + N(x)); }
int add(int a, int b) { return node("+ " + N(a) + " " + N(b)); }
int neg(int a) { return node("- " + N(a)); }
int con(int a, const string& c) { return node("C " + N(a) + " " + c); }
int shl(int a, int k) { return node("< " + N(a) + " " + N(k)); }
int sig(int a) { return node("S " + N(a)); }
int cmp(int a, int b) { return node("P " + N(a) + " " + N(b)); }
int mx(int a, int b) { return node("M " + N(a) + " " + N(b)); }
int mul(int a, int b) { return node("* " + N(a) + " " + N(b)); }
string pow2(int i) { return to_string(1ULL << i); }
// 1 if x >= 0, else 0, for x a half-integer
int stepHalf(int x) { return sig(shl(x, 10)); }

// bits of an integer 0 <= a < 2^32, high to low
vector<int> bits(int a) {
    vector<int> b;
    int r = a;
    for (int i = 31; i >= 0; i--) {
        int g = stepHalf(con(r, "-" + to_string((1ULL << i) - 1) + ".5"));  // r - 2^i + 1/2
        b.push_back(g);
        if (i) r = add(r, neg(shl(g, i)));
    }
    return b;
}
int horner(const vector<int>& b) {
    int x = b[0];
    for (size_t i = 1; i < b.size(); i++) x = add(shl(x, 1), b[i]);
    return x;
}

int main() {
    int t;
    scanf("%d", &t);
    if (t == 1) {
        int a = I(), b = I();
        O(neg(shl(add(a, b), 1)));
    } else if (t == 2) {
        int a = I();
        O(sig(neg(add(shl(a, 4), a))));
    } else if (t == 3) {
        int a = I();
        O(cmp(a, add(a, neg(a))));
    } else if (t == 4) {
        int a = I();
        O(mx(a, neg(a)));
    } else if (t == 5) {
        vector<int> a(32);
        for (auto& x : a) x = I();
        O(horner(a));
    } else if (t == 6) {
        int a = I();
        for (int g : bits(a)) O(g);
    } else if (t == 7) {
        int a = I(), b = I();
        auto x = bits(a), y = bits(b);
        vector<int> z(32);
        for (int i = 0; i < 32; i++) {
            int s = add(x[i], y[i]);  // xor = s - 2 [s >= 2]
            z[i] = add(s, neg(shl(stepHalf(con(s, "-1.5")), 1)));
        }
        O(horner(z));
    } else if (t == 8) {
        int a = I();
        O(mul(a, con(add(a, neg(a)), "0.1")));
    } else if (t == 9) {
        vector<int> a(16);
        for (auto& x : a) x = I();
        for (int r = 0; r < 16; r++)
            for (int i = r % 2; i + 1 < 16; i += 2) {
                int hi = mx(a[i], a[i + 1]);
                int lo = neg(mx(neg(a[i]), neg(a[i + 1])));
                a[i] = lo, a[i + 1] = hi;
            }
        for (int x : a) O(x);
    } else {
        int a = I(), b = I(), m = I();
        int r = mul(a, b);
        for (int i = 63; i >= 0; i--) {
            int d = shl(m, i);
            int g = stepHalf(con(cmp(r, d), "0.5"));  // r >= m 2^i
            r = add(r, neg(mul(g, d)));
        }
        O(r);
    }
    for (auto& s : out) puts(s.c_str());
}
