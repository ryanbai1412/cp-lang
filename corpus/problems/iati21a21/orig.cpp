// IATI 2021 Hint. No accepted submission exists (the four C++ ones crash). genHint marks,
// for each position of A, whether the leftmost embedding of the given optimal C uses it
// (N bits); solve prints the marked elements of A. The main stands in for the grader's two
// rounds as interactor.py runs them: N M, A, B, K, C -> L and the bits; N M, A, B, L, bits ->
// the length and the sequence.
#include <bits/stdc++.h>
using namespace std;

vector<bool> genHint(const vector<int>& a, const vector<int>&, const vector<int>& sol) {
    vector<bool> hint(a.size());
    size_t k = 0;
    for (size_t i = 0; i < a.size() && k < sol.size(); i++)
        if (a[i] == sol[k]) hint[i] = true, k++;
    return hint;
}

vector<int> solve(const vector<int>& a, const vector<int>&, const vector<bool>& hint) {
    vector<int> out;
    for (size_t i = 0; i < a.size(); i++)
        if (hint[i]) out.push_back(a[i]);
    return out;
}

static vector<int> readv(int n) {
    vector<int> v(n);
    for (int& x : v)
        if (scanf("%d", &x) != 1) exit(1);
    return v;
}

int main() {
    int n, m, k;
    if (scanf("%d %d", &n, &m) != 2) return 1;
    vector<int> a = readv(n), b = readv(m);
    if (scanf("%d", &k) != 1) return 1;
    vector<bool> hint = genHint(a, b, readv(k));
    string bits(hint.size(), '0');
    for (size_t i = 0; i < hint.size(); i++) bits[i] = hint[i] ? '1' : '0';
    printf("%zu\n%s\n", bits.size(), bits.c_str());
    fflush(stdout);
    int l;
    if (scanf("%d %d", &n, &m) != 2) return 1;
    a = readv(n), b = readv(m);
    if (scanf("%d", &l) != 1) return 1;
    static char buf[1 << 20];
    vector<bool> got(l);
    if (l) {
        string s;
        while ((int)s.size() < l && scanf("%1048575s", buf) == 1) s += buf;
        for (int i = 0; i < l; i++) got[i] = s[i] == '1';
    }
    vector<int> c = solve(a, b, got);
    printf("%zu\n", c.size());
    for (int x : c) printf("%d ", x);
    printf("\n");
}
