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
	int n, q;
	cin >> n >> q;
	long pa[n + 1] {}, pb[n + 1] {}, pc[n+1]{};
	while(q--) {
		long t, l, r, a, b;
		cin >> t >> l >> r >> a >> b;
		l--;
		r--;
		if(t == 1) {
			a += (r - l + 2) * b;
			b *= -1;
		}
		pa[l] += a;
		pa[r + 1] -= a;
		pb[l] += b;
		pb[r + 1] -= b;
		pc[r + 1] -= b * (r - l + 1);
	}
	long sum = 0, cur = 0, bsum = 0;
	for(int i = 0; i < n; i++) {
		cur += pa[i];
		sum += pb[i];
		bsum += sum + pc[i];
		cout << cur + bsum << endl;
	}
}