#include "bits/stdc++.h"

const int maxn = 1024;

char buf[maxn];

int main() {
	auto first = fread(buf, 1, sizeof(buf), stdin);
	int ind = 0;
	while(buf[ind++] != '\n');
	fwrite(buf + ind, 1, first - ind, stdout);
	size_t read;
	while((read = fread(buf, 1, sizeof(buf), stdin))) {
		fwrite(buf, 1, sizeof(buf), stdout);
	}
}