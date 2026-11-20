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
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	long n, m;
	cin >> n >> m;
	int k;
	cin >> k;
	cout << k << endl;
	for(int i = 0; i < k; i++) {
		long x, y;
		cin >> x >> y;
		cout << x << " " << y << endl;
	}
}