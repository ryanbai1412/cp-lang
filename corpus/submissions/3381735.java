//package Mock_CCC_Senior21;

import java.util.*;
import java.io.*;

import static java.lang.Math.*;

public class Problem3 {
	static class Input {
		final byte[] buff = System.in.readAllBytes();
		int ind = 0;
		public Input() throws IOException {
		}
		public int nextInt() throws IOException {
			byte b;
			while((b = buff[ind++])<'0'||b>'9') ;
			int ret = b-'0';
			while((b = buff[ind++])>='0'&&b<='9') {
				ret = ret*10+(b-'0');
			}
			return ret;
		}
	}

	public static void main(String[] args) throws Exception {
		long start = System.currentTimeMillis();
		Input in = new Input();
		PrintWriter pw = new PrintWriter(System.out);
		new Problem3().solve(in, pw);
		pw.close();
		long end = System.currentTimeMillis();
		System.err.println(end-start);
	}

	static final boolean LOCAL = false;

	public static void dbg(Object... o) {
		if(LOCAL) {
			System.err.println(Arrays.deepToString(o));
		}
	}

	static class ST {
		int n, ind, x;
		int[] arr, v;
		public ST(int[] arr) {
			this.n = arr.length;
			this.arr = arr;
			v = new int[4*n];
			build(1, 0, n);
		}
		public void build(int o, int l, int r) {
			if(l+1==r) {
				v[o] = arr[l];
			}else {
				int mid = (l+r)/2, lc = o*2, rc = lc+1;
				build(lc, l, mid);
				build(rc, mid, r);
				v[o] = min(v[lc], v[rc]);
			}
		}
		public void set(int o, int l, int r) {
			if(l+1==r) {
				v[o] = x;
			}else {
				int mid = (l+r)/2, lc = o*2, rc = lc+1;
				if(ind<mid) {
					set(lc, l, mid);
				}else {
					set(rc, mid, r);
				}
				v[o] = min(v[lc], v[rc]);
			}
		}
		public void set(int ind, int x) {
			dbg(ind, x);
			this.ind = ind;
			this.x = x;
			set(1, 0, n);
		}
		public void query(int o, int l, int r) {
			if(l+1==r) {
				ind = l;
			}else {
				int mid = (l+r)/2, lc = o*2, rc = lc+1;
				if(v[lc]<=x) {
					query(lc, l, mid);
				}else {
					query(rc, mid, r);
				}
			}
		}
		public int query(int x) {
			this.x = x;
			query(1, 0, n);
			return ind;
		}
	}

	public void solve(Input in, PrintWriter pw) throws Exception {
		int n = in.nextInt(), k = in.nextInt();
		ArrayDeque<Integer>[] ind = new ArrayDeque[k];
		for(int i = 0; i<k; i++) {
			ind[i] = new ArrayDeque<>();
		}
		int[] arr = new int[n];
		for(int i = 0; i<n; i++) {
			ind[arr[i] = in.nextInt()-1].addLast(i);
		}
		int[][] last = new int[k][2];
		for(int i = 0; i<k; i++) {
			last[i][0] = ind[i].getLast();
			last[i][1] = i;
		}
		Arrays.sort(last, Comparator.comparingInt(o -> o[0]));
		boolean[] removed = new boolean[k];
		ST st;
		{
			int[] tmp = new int[k];
			for(int i = 0; i<k; i++) {
				tmp[i] = ind[i].getFirst();
			}
			st = new ST(tmp);
		}
		int prev = 0;
		StringBuilder sb = new StringBuilder(k*4);
		for(int[] i: last) {
			if(!removed[i[1]]) {
				while(true) {
					int cur = st.query(i[0]);
					sb.append(" ").append(cur+1);
					removed[cur] = true;
					st.set(cur, n+1);
					int time = ind[cur].getFirst();
					for(int j = prev; j<=time; j++) {
						ind[arr[j]].removeFirst();
						if(!removed[arr[j]]) {
							if(ind[arr[j]].isEmpty()) {
								st.set(arr[j], n+1);
							}else {
								st.set(arr[j], ind[arr[j]].getFirst());
							}
						}
					}
					prev = time+1;
					if(cur==i[1]) {
						break;
					}
				}
			}
		}
		System.out.println(sb.toString());
	}
}