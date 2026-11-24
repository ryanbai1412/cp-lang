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
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif
 
#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())
 
// The original hands the circuit to the IOI grader's answer. This stand-in prints it:
// C[0..=M], then X and Y, each led by its length; `main` reads M n, then A[0..n).
void answer(vector<int> C, vector<int> X, vector<int> Y) {
	for (size_t i = 0; i < C.size(); i++) printf("%d%c", C[i], i + 1 < C.size() ? ' ' : '\n');
	for (auto* v : {&X, &Y}) {
		printf("%d", int(v->size()));
		for (int x : *v) printf(" %d", x);
		printf("\n");
	}
}
 
vector<int> arr;
vector<pair<int, int>> ans;
 
const int none = -1e9;
 
int n, pref, root = none;
 
void init(int& o, int l, int r, int i) {
    if (r < pref) {
        o = root;
        return;
    } else if (l == r) {
        o = arr.back();
        arr.pop_back();
        return;
    }
    if (o == none) {
        o = -sz(ans) - 1;
        ans.emplace_back(none, none);
    }
    int mid = (l + r) / 2;
    if (i % 2 == 0) {
        init(ans[-o - 1].first, l, mid, i >> 1);
    } else {
        init(ans[-o - 1].second, mid + 1, r, i >> 1);
    }
}
 
void create_circuit(int m, vector<int> _arr) {
    arr = _arr;
    int first = arr[0];
    arr.erase(arr.begin());
    arr.push_back(0);
    reverse(begin(arr), end(arr));
 
    n = sz(arr);
    int targ = 1 << (__lg(n - 1) + 1);
    pref = targ - n;
    ans.reserve(targ * 5);
    for (int i = 0; i < targ; i++) {
        init(root, 0, targ - 1, i);
    }
    vector<int> ans1(m + 1, root);
    ans1[0] = first;
    vector<int> ans2, ans3;
    for (auto& [x, y] : ans) {
        ans2.push_back(x);
        ans3.push_back(y);
    }
    answer(ans1, ans2, ans3);
}
int main() {
	int m, n;
	if (scanf("%d %d", &m, &n) != 2) return 1;
	vector<int> a(n);
	for (auto& v : a) scanf("%d", &v);
	create_circuit(m, a);
}
