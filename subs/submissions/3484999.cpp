#include "bits/stdc++.h"

using namespace std;

//sad; long long double doesn't exist
using ld = long double;

//imagine a language where int = long
#define long long long

//typing too hard
#define endl "\n"

#define sz(x) int((x).size())

int main() {
	//bruh mfw ac gives me 10/100
	//ok I can't read
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	long n, m;
	cin >> n >> m;
	map<int, int> mx, my;
	int k;
	cin >> k;
	for(int i = 0; i < k; i++) {
		long x, y;
		cin >> x >> y;
		mx[x] ^= 1;
		my[y] ^= 1;
	}
	vector<int> x, y;
	for(auto &[u, v]: mx) {
		if(v) {
			x.push_back(u);
		}
	}
	for(auto &[u, v]: my) {
		if(v) {
			y.push_back(v);
		}
	}
	cout << max(sz(x), sz(y)) << endl;
	int ind = 0;
	while(ind < min(sz(x), sz(y))) {
		cout << x[ind] << " " << y[ind] << endl;
		ind++;
	}
	while(ind < sz(x)) {
		cout << x[ind] << " 1" << endl;
		ind++;
	}
	while(ind < sz(y)) {
		cout << y[ind] << " 1" << endl;
		ind++;
	}
}