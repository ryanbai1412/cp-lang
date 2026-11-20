import java.io.*;
import java.util.*;

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
			RiolkusMockCCCS2KeenKeenerSequence solver = new RiolkusMockCCCS2KeenKeenerSequence();
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

	static class RiolkusMockCCCS2KeenKeenerSequence {
		public RiolkusMockCCCS2KeenKeenerSequence() {
		}

		public void solve(int kase, InputReader in, Output pw) {
			int n = in.nextInt();
			long[] arr = in.nextLong(n);
			Utilities.sort(arr);
			long ans = 0;
			MultiHashSet<Long> tot = new MultiHashSet<>();
			for(int i = 0; i<n; i++) {
				tot.add(arr[i]);
				MultiHashSet<Long> hs = new MultiHashSet<>();
				for(int j = i+1; j<n; j++) {
					ans += hs.count(arr[i]*arr[j]);
					for(int k = i+1; k<j; k++) {
						hs.add(arr[k]*arr[j]);
					}
				}
			}
			for(long i: tot.values()) {
				long ways = i*(i-1)*(i-2)*(i-3)/24;
				ans += ways*2;
			}
			for(var v: tot.entrySet()) {
				for(var w: tot.entrySet()) {
					if(v.getKey()<w.getKey()) {
						long cur = Math.min(v.getValue(), w.getValue());
						long ways = cur*(cur-1)/2;
						ans += ways;
					}
				}
			}
			pw.println(ans*8);
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

		public void println(long l) {
			println(String.valueOf(l));
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

	interface InputReader {
		int nextInt();

		long nextLong();

		default long[] nextLong(int n) {
			long[] ret = new long[n];
			for(int i = 0; i<n; i++) {
				ret[i] = nextLong();
			}
			return ret;
		}

	}

	static class MultiHashSet<T> extends HashMap<T, Integer> implements MultiSet<T> {
	}

	static class FastReader implements InputReader {
		final private int BUFFER_SIZE = 1<<16;
		private final DataInputStream din;
		private final byte[] buffer;
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

		public long nextLong() {
			long ret = 0;
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

	interface MultiSet<T> extends Map<T, Integer> {
		default int count(T key) {
			Integer cnt = get(key);
			return cnt==null ? 0 : cnt;
		}

		default void add(T key) {
			add(key, 1);
		}

		default void add(T key, int count) {
			if(count<0) {
				delete(key, -count);
			}else if(count>0) {
				compute(key, (k, v) -> {
					if(v==null) {
						v = 0;
					}
					return v+count;
				});
			}
		}

		default void delete(T key, int count) {
			if(count<0) {
				add(key, -count);
			}else if(count>0) {
				compute(key, (k, v) -> {
					if(v==null) {
						v = 0;
					}
					v -= count;
					return v<=0 ? null : v;
				});
			}
		}

	}

	static class Utilities {
		public static void sort(long[] arr) {
			Random rand = new Random();
			int n = arr.length;
			for(int i = 0; i<n; i++) {
				swap(arr, i, rand.nextInt(n));
			}
			Arrays.sort(arr);
		}

		public static void swap(long[] arr, int i, int j) {
			if(i!=j) {
				arr[i] ^= arr[j];
				arr[j] ^= arr[i];
				arr[i] ^= arr[j];
			}
		}

	}
}