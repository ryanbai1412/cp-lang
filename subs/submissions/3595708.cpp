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

const int maxv = (1 << 31) - 1;

void solve() {
	int n, m;
	cin >> n >> m;
	int arr[n][2];
	int a = 0, b = maxv;
	for(int i = 0; i < n; i++) {
		string s;
		cin >> s >> arr[i][1];
		if(s == "AND") {
			arr[i][0] = 0;
			a &= arr[i][1];
			b &= arr[i][1];
		}else if(s == "XOR") {
			arr[i][0] = 1;
			a ^= arr[i][1];
			b ^= arr[i][1];
		}else {
			arr[i][0] = 2;
			a |= arr[i][1];
			b |= arr[i][1];
		}
	}
	int cur = 0;
	bool fill = false;

	auto get = [] (const int &mask, int bit) {
		return (mask >> bit) & 1;
	};
	for(int i = 30; i >= 0; i--) {
		int limit = get(m, i);
		int av = get(a, i), bv = get(b, i);
		dbg(av, bv);
		if(fill) {
			limit = 1;
		}
		cur <<= 1;
		if(limit == 0) {
			continue;
		}
		if(av == 0) {
			if(bv == 0) {
				// always 0
				fill = true;
			}else {
				// equal to you
				cur |= 1;
			}
		}else {
			fill = true;
			cur |= 1;
		}
	}
	cout << cur << endl;
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