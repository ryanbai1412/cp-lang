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
	int n, m;
	cin >> n >> m;
	for(int a = 0; a < n; a++) {
		if(a == 0) {
			for(int i = 0; i < m; i++) {
				cout << "B";
			}
		}else if(a == 1) {
			for(int i = 0; i < m; i++) {
				cout << "Y";
			}
		}else if(a == 2) {
			for(int i = 0; i < m; i++) {
				cout << "BY"[i & 1];
			}
		}else {
			for(int i = 0; i < m; i++) {
				cout << "YB"[i & 1];
			}
		}
		cout << endl;
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