#include "bits/extc++.h"

using namespace std;

bool query(int);
void modify(int, bool);

bool WereYouLast(int n, int) {
	n = __lg(n);
	if (n == 10) {
		int i;
		for (i = 0; i < n; i++) {
			if (query(i + 1)) {
				modify(i + 1, false);
			} else {
				modify(i + 1, true);
				break;
			}
		}
		if (i == n) {
			return true;
		}
		return false;
	}
	assert(false);
}