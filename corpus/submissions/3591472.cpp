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
	string s;
	cin >> s;
	int q;
	cin >> q;
	int last = 0;
	while(q--) {
		int l, r;
		cin >> l >> r;
		l ^= last;
		r ^= last;
		l--;
		r--;
		set<string> ans;
		for(int i = l; i <= r; i++) {
			for(int j = i; j <= r; j++) {
				ans.insert(s.substr(i, j - i + 1));
			}
		}
		last = 0;
		for(auto &a: ans) {
			last += sz(a) + 1;
		}
		cout << last << endl;
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