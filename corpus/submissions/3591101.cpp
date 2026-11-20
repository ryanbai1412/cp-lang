#include "bits/extc++.h"

using namespace std;

template<class T, class U>
void sep(T &out, const string &s, const U &u) {
	out << u;
}

template<class T, class Head, class ...Tail>
void sep(T &out, const string &s, const Head &h, const Tail &...t) {
	out << h << s;
	sep(out, s, t...);
}

#ifdef DEBUG
#define dbg(...)                                                      \
cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" << ": ";       \
sep(cerr, " | ", __VA_ARGS__);                                        \
cerr << endl
#else
#define cerr if(false) cerr
#define dbg(...) cerr
#endif

//imagine a language where int = long
#define long int64_t

//typing too hard
#define endl "\n"

#define sz(x) int((x).size())

void solve() {
	long n, m;
	cin >> n >> m;
	set<pair<long, long>> ans;
	for(long i = 0; i <= m; i++) {
		long cur = n * i;
		if(cur % m != 0) {
			ans.emplace(cur / m, i - 1);
			ans.emplace(cur / m, i);
		}
	}
	for(long i = 0; i <= n; i++) {
		long cur = m * i;
		if(cur % n != 0) {
			ans.emplace(i - 1, cur / n);
			ans.emplace(i, cur / n);
		}
	}
	cout << sz(ans) << endl;
	for(auto &a: ans) {
		assert(0 <= a.first && a.first < n);
		assert(0 <= a.second && a.second < m);
		cout << a.first + 1 << " " << a.second + 1 << endl;
	}
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
//	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}