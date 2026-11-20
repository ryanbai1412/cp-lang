import java.io.BufferedOutputStream;
import java.io.Closeable;
import java.io.DataInputStream;
import java.io.Flushable;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collection;
import java.util.InputMismatchException;
import java.util.function.BiFunction;
import java.util.function.BinaryOperator;
import java.util.stream.BaseStream;
import java.util.stream.Collectors;
import java.util.stream.Stream;

/**
 * Built using CHelper plug-in
 * Actual solution is at the top
 */
public class Main {
	static class TaskAdapter implements Runnable {
		@Override
		public void run() {
			long startTime = System.currentTimeMillis();
			InputStream inputStream = System.in;
			OutputStream outputStream = System.out;
			FastReader in = new FastReader(inputStream);
			Output out = new Output(outputStream);
			RiolkusMockCCCS5KeenKeenerMultiset solver = new RiolkusMockCCCS5KeenKeenerMultiset();
			solver.solve(1, in, out);
			out.close();
			System.err.println(System.currentTimeMillis()-startTime+"ms");
		}
	}

	public static void main(String[] args) throws Exception {
		Thread thread = new Thread(null, new TaskAdapter(), "", 1<<29);
		thread.start();
		thread.join();
	}

	static class RiolkusMockCCCS5KeenKeenerMultiset {
		static final int m = 500000;

		public RiolkusMockCCCS5KeenKeenerMultiset() {
		}

		public void solve(int kase, InputReader in, Output pw) {
			int n = in.nextInt(), q = in.nextInt();
			if(q<=2000) {
				ArrayList<Integer> set = new ArrayList<>();
				for(int i = 0; i<n; i++) {
					set.add(in.nextInt());
				}
				while(q-->0) {
					int type = in.nextInt();
					if(type==1) {
						set.add(in.nextInt());
					}else if(type==2) {
						int l = in.nextInt(), r = in.nextInt(), x = in.nextInt();
						set = set.stream().parallel().map(o -> (o>=l&&o<=r) ? (o^x) : o).collect(Collectors.toCollection(ArrayList::new));
					}else {
						int l = in.nextInt(), r = in.nextInt();
						pw.println(set.stream().parallel().filter(o -> o>=l&&o<=r).reduce(0, (a, b) -> a^b));
					}
				}
			}else {
				Integer[] arr = new Integer[m+1];
				Arrays.fill(arr, 0);
				for(int i = 0; i<n; i++) {
					int x = in.nextInt();
					arr[x] ^= x;
				}
				TPointSegmentTree<Integer> st = new TPointSegmentTree<>(arr, (a, b) -> a^b);
				while(q-->0) {
					int type = in.nextInt();
					if(type==1) {
						int x = in.nextInt();
						st.set(x, arr[x] ^= x);
					}else if(type==3) {
						pw.println(st.query(in.nextInt(), in.nextInt()));
					}else {
						throw new AssertionError();
					}
				}
			}
		}

	}

	static class Output implements Closeable, Flushable {
		public StringBuilder sb;
		public OutputStream os;
		public int BUFFER_SIZE;
		public String lineSeparator;

		public Output(OutputStream os) {
			this(os, 1<<16);
		}

		public Output(OutputStream os, int bs) {
			BUFFER_SIZE = bs;
			sb = new StringBuilder(BUFFER_SIZE);
			this.os = new BufferedOutputStream(os, 1<<17);
			lineSeparator = System.lineSeparator();
		}

		public void println(int i) {
			println(String.valueOf(i));
		}

		public void println() {
			sb.append(lineSeparator);
		}

		public void println(String s) {
			sb.append(s);
			println();
		}

		private void flushToBuffer() {
			try {
				os.write(sb.toString().getBytes());
			}catch(IOException e) {
				e.printStackTrace();
			}
			sb = new StringBuilder(BUFFER_SIZE);
		}

		public void flush() {
			try {
				flushToBuffer();
				os.flush();
			}catch(IOException e) {
				e.printStackTrace();
			}
		}

		public void close() {
			flush();
			try {
				os.close();
			}catch(IOException e) {
				e.printStackTrace();
			}
		}

	}

	static interface InputReader {
		int nextInt();

	}

	static class TPointSegmentTree<T> {
		public int n;
		public int ind;
		public int ql;
		public int qr;
		public T[] arr;
		public T[] value;
		public BinaryOperator<T> operation;

		public TPointSegmentTree(T[] arr, BinaryOperator<T> operation) {
			n = arr.length;
			this.arr = arr;
			value = (T[]) new Object[n<<2];
			this.operation = operation;
			build(1, 0, n-1);
		}

		public TPointSegmentTree(int n, T initialValue, BinaryOperator<T> operation) {
			this.n = n;
			arr = (T[]) new Object[n];
			value = (T[]) new Object[n<<2];
			Arrays.fill(arr, initialValue);
			this.operation = operation;
			build(1, 0, n-1);
		}

		private void build(int o, int l, int r) {
			if(l==r) {
				value[o] = arr[l];
				return;
			}
			int lc = o<<1, rc = lc|1, mid = l+r >> 1;
			build(lc, l, mid);
			build(rc, mid+1, r);
			value[o] = operation.apply(value[lc], value[rc]);
		}

		private void set(int o, int l, int r) {
			if(l==r) {
				value[o] = arr[l];
				return;
			}
			int lc = o<<1, rc = lc|1, mid = l+r >> 1;
			if(ind<=mid) {
				set(lc, l, mid);
			}else {
				set(rc, mid+1, r);
			}
			value[o] = operation.apply(value[lc], value[rc]);
		}

		public void set(int ind, T val) {
			this.ind = ind;
			arr[ind] = val;
			set(1, 0, n-1);
		}

		private T query(int o, int l, int r) {
			if(ql<=l&&qr>=r) {
				return value[o];
			}
			int lc = o<<1, rc = lc|1, mid = l+r >> 1;
			T ret = null;
			if(ql<=mid) {
				ret = query(lc, l, mid);
			}
			if(qr>mid) {
				if(ret==null) {
					ret = query(rc, mid+1, r);
				}else {
					ret = operation.apply(ret, query(rc, mid+1, r));
				}
			}
			return ret;
		}

		public T query(int l, int r) {
			ql = l;
			qr = r;
			return query(1, 0, n-1);
		}

	}

	static class FastReader implements InputReader {
		final private int BUFFER_SIZE = 1<<16;
		private DataInputStream din;
		private byte[] buffer;
		private int bufferPointer;
		private int bytesRead;

		public FastReader(InputStream is) {
			din = new DataInputStream(is);
			buffer = new byte[BUFFER_SIZE];
			bufferPointer = bytesRead = 0;
		}

		public int nextInt() {
			int ret = 0;
			byte c = skipToDigit();
			boolean neg = (c=='-');
			if(neg) {
				c = read();
			}
			do {
				ret = ret*10+c-'0';
			} while((c = read())>='0'&&c<='9');
			if(neg) {
				return -ret;
			}
			return ret;
		}

		private boolean isDigit(byte b) {
			return b>='0'&&b<='9';
		}

		private byte skipToDigit() {
			byte ret;
			while(!isDigit(ret = read())&&ret!='-') ;
			return ret;
		}

		private void fillBuffer() {
			try {
				bytesRead = din.read(buffer, bufferPointer = 0, BUFFER_SIZE);
			}catch(IOException e) {
				e.printStackTrace();
				throw new InputMismatchException();
			}
			if(bytesRead==-1) {
				buffer[0] = -1;
			}
		}

		private byte read() {
			if(bytesRead==-1) {
				throw new InputMismatchException();
			}else if(bufferPointer==bytesRead) {
				fillBuffer();
			}
			return buffer[bufferPointer++];
		}

	}
}