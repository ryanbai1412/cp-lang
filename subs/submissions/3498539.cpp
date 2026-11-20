#include "unistd.h"

const int maxn = 1 << 16;

char buf[maxn];

int main() {
	setvbuf(stdin, nullptr, _IONBF, 0);
	setvbuf(stdout, nullptr, _IONBF, 0);
	while(getchar_unlocked() != '\n');
	ssize_t rcnt;
	while((rcnt = read(stdin, buf, maxn))) {
		write(stdout, buf, rcnt);
	}
}