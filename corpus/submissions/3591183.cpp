#ifdef __CLION_IDE__
#include "bits/stdc++.h"
#else
#include "bits/extc++.h"
#endif

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
	vector<vector<int>> arr(n, vector<int>(m));
	for(int i = 0; i < n; i++) {
		string s;
		cin >> s;
		for(int j = 0; j < m; j++) {
			arr[i][j] = s[j] == 'B';
		}
	}
	if(n > m) {
		vector<vector<int>> narr(m, vector<int>(n));
		for(int i = 0; i < n; i++) {
			for(int j = 0; j < m; j++) {
				narr[j][i] = arr[i][j];
			}
		}
		arr = narr;
		swap(n, m);
	}
	long ans = 0;
	for(int i = 0; i < n; i++) {
		vector<vector<int>> cdiff;
		{
			vector<int> a, b;
			for(int j = 0; j < m; j++) {
				if(arr[i][j] == 0) {
					a.push_back(j);
				}else {
					b.push_back(j);
				}
			}
			if(sz(a)) {
				cdiff.push_back(a);
			}
			if(sz(b)) {
				cdiff.push_back(b);
			}
		}
		for(int j = i + 1; j < n; j++) {
			if(j > i) {
				vector<vector<int>> ndiff;
				for(auto &x: cdiff) {
					{
						vector<int> a, b;
						for(auto &k: x) {
							if(arr[j][k] == 0) {
								a.push_back(k);
							}else {
								b.push_back(k);
							}
						}
						if(sz(a)) {
							ndiff.push_back(a);
						}
						if(sz(b)) {
							ndiff.push_back(b);
						}
					}
				}
				cdiff = ndiff;
			}
			int dend[m + 1];
			dend[m] = m - 1;
			for(int k = m - 1; k >= 0; k--) {
				dend[k] = dend[k + 1];
				if(arr[i][k] != arr[j][k]) {
					dend[k] = k - 1;
				}
			}
			for(auto &a: cdiff) {
				for(int k = 0; k < sz(a); k++) {
					if(dend[a[k]] > a[k]) {
						int cur = (upper_bound(begin(a), end(a), dend[a[k]]) - a.begin()) - k - 1;
						if(cur) {
							dbg(i, j, a[k], dend[a[k]], cur);
						}
						ans += cur;
					}
				}
			}
		}
	}
	cout << ans << endl;
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