#include "bits/stdc++.h"

const int maxn = 1024;

char buf[maxn];

int main() {
	while(fgetc(stdin) != '\n');
	size_t read;
	while((read = fread(buf, 1, sizeof(buf), stdin))) {
		fwrite(buf, 1, read, stdout);
	}
}