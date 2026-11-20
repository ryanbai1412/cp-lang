pub use __cargo_equip::prelude::*;

use lib::prelude::*;

#[derive(Default)]
struct Solver;

const MAXN: usize = 5e6 as usize + 5;

static mut ARR_R: [bool; MAXN] = [false; MAXN];
static mut ARR_C: [bool; MAXN] = [false; MAXN];

impl StandardSolver for Solver {
    fn solve<C: ReadExtConfig>(self, inp: &mut impl ReadExt<C>, out: &mut impl WriteExt) {
        let (n, m, k) = inp.read::<(usize, usize, usize)>();
        let arr_r = unsafe { &mut ARR_R[..n] };
        let arr_c = unsafe { &mut ARR_C[..m] };

        for _ in 0..k {
            let (c, mut ind) = inp.read::<(char, usize)>();
            ind -= 1;

            if c == b'R' {
                arr_r[ind] ^= true;
            } else if c == b'C' {
                arr_c[ind] ^= true;
            } else {
                panic!()
            }
        }

        fn cnt_occ(arr: &[bool]) -> (usize, usize) {
            let cnt_false = arr.iter().filter(|&x| !x).count();
            (cnt_false, arr.len() - cnt_false)
        }

        let (r_f, r_t) = cnt_occ(arr_r);
        let (c_f, c_t) = cnt_occ(arr_c);
        writeln!(out, "{}", r_f * c_t + r_t * c_f).unwrap();
    }
}

fn main() {
    solver::main::<Solver, io::config::NoOverflowChecks>(ProblemTestcaseType::SingleTest)
}

// The following code was expanded by `cargo-equip`.

///  # Bundled libraries
/// 
///  - `lib 0.1.0 (path+████████████████████████████████████████████████)` published in **missing** licensed under **missing** as `crate::__cargo_equip::crates::lib`
#[cfg_attr(any(), rustfmt::skip)]
#[allow(unused)]
mod __cargo_equip {
    pub(crate) mod crates {
        pub mod lib {#![allow(unstable_name_collisions)]pub mod io{#![doc=" IO stuff"]#![doc=""]#![doc=" By default, all inputs are delimited by [ascii whitespace](u8::is_ascii_whitespace) and consume"]#![doc=" the trailing whitespace upon being read."]#![doc=""]#![doc=" The behavior of this module is undefined if the input is not valid UTF-8."]use super::nightly_ext::*;use std::{io::{BufReader,BufWriter,Read,Write},iter::Peekable,mem::MaybeUninit,str::FromStr,};pub mod prelude{pub use super::{ReadExt,ReadExtConfig,StringLine,WriteExt};}pub mod config{pub trait ReadExtConfig{const OVERFLOW_CHECKS:bool=true;}impl ReadExtConfig for(){}pub struct NoOverflowChecks;impl ReadExtConfig for NoOverflowChecks{const OVERFLOW_CHECKS:bool=false;}}pub use config::ReadExtConfig;pub fn from_read<R:Read>(r:R)->impl ReadExt<()>{from_read_with_config::<R,()>(r)}pub fn from_read_with_config<R:Read,C:ReadExtConfig>(r:R)->impl ReadExt<C>{BufReader::with_capacity(1<<16,r).bytes().map(|c|unsafe{c.unwrap_unchecked()}).peekable()}pub fn stdin()->impl ReadExt<()>{stdin_with_config::<()>()}pub fn stdin_with_config<C:ReadExtConfig>()->impl ReadExt<C>{let stdin=std::io::stdin().lock();from_read_with_config::<_,C>(stdin)}pub fn stdout()->impl WriteExt{BufWriter::with_capacity(1<<16,std::io::stdout().lock())}pub trait Readable{type Output;fn read_from<C:ReadExtConfig>(r:&mut impl ReadExt<C>)->Self::Output;}pub trait ReadExt<C:ReadExtConfig>{fn next_byte(&mut self)->Option<u8>;fn peek_byte(&mut self)->Option<u8>;fn skip_while<F:FnMut(&u8)->bool>(&mut self,f:F);fn read<T:Readable>(&mut self)->T::Output;fn read_vec<T:Readable>(&mut self,n:usize)->Vec<T::Output>;#[doc=" Advances `self` until either a non-whitespace character is found, or eof is reached."]#[doc=" Returns `true` if eof is reached."]fn eof(&mut self)->bool;}pub trait WriteExt:Write{}impl<T:Write>WriteExt for T{}impl<C:ReadExtConfig,I:Iterator<Item=u8>>ReadExt<C>for Peekable<I>{fn next_byte(&mut self)->Option<u8>{self.next()}fn peek_byte(&mut self)->Option<u8>{self.peek().copied()}fn skip_while<F:FnMut(&u8)->bool>(&mut self,mut f:F){while self.next_if(|c|f(c)).is_some(){}}fn read<T:Readable>(&mut self)->T::Output{T::read_from::<C>(self)}fn read_vec<T:Readable>(&mut self,n:usize)->Vec<T::Output>{(0..n).map(|_|<Self as ReadExt<C>>::read::<T>(self)).collect()}fn eof(&mut self)->bool{ReadExt::<C>::skip_while(self,u8::is_ascii_whitespace);<Self as ReadExt<C>>::peek_byte(self).is_none()}}impl Readable for char{type Output=u8;fn read_from<C:ReadExtConfig>(r:&mut impl ReadExt<C>)->Self::Output{r.skip_while(u8::is_ascii_whitespace);r.next_byte().expect("eof")}}impl Readable for String{type Output=Vec<u8>;fn read_from<C:ReadExtConfig>(r:&mut impl ReadExt<C>)->Self::Output{r.skip_while(u8::is_ascii_whitespace);let mut buf=vec![];while let Some(c)=r.next_byte(){if c.is_ascii_whitespace(){break;}buf.push(c);}assert!(!buf.is_empty(),"eof");buf}}macro_rules!uint_readable_impl{($ty:ty)=>{impl Readable for$ty{type Output=$ty;fn read_from<C:ReadExtConfig>(r:&mut impl ReadExt<C>)->Self::Output{r.skip_while(u8::is_ascii_whitespace);r.peek_byte().expect("eof");let mut ans=0;while let Some(c)=r.next_byte(){if c.is_ascii_whitespace(){break;}assert!(c.is_ascii_digit());let c=(c-b'0')as$ty;if C::OVERFLOW_CHECKS{if ans<=(<$ty>::MAX-c)/10{ans=ans*10+c;}else{panic!(concat!("overflow when reading ",stringify!($ty)))}}else{ans=ans*10+c;}}ans}}};($($ty:ty),*)=>{$(uint_readable_impl!($ty);)*};}macro_rules!sint_readable_impl{($ty:ty)=>{impl Readable for$ty{type Output=$ty;fn read_from<C:ReadExtConfig>(r:&mut impl ReadExt<C>)->Self::Output{r.skip_while(u8::is_ascii_whitespace);if r.peek_byte().expect("eof")==b'-'{r.next_byte();let mut ans=0;while let Some(c)=r.next_byte(){if c.is_ascii_whitespace(){break;}assert!(c.is_ascii_digit());let c=(c-b'0')as$ty;if C::OVERFLOW_CHECKS{if(<$ty>::MIN+c)/10<=ans{ans=ans*10-c;}else{panic!(concat!("underflow when reading ",stringify!($ty)))}}else{ans=ans*10-c;}}ans}else{let mut ans=0;while let Some(c)=r.next_byte(){if c.is_ascii_whitespace(){break;}assert!(c.is_ascii_digit());let c=(c-b'0')as$ty;if C::OVERFLOW_CHECKS{if ans<=(<$ty>::MAX-c)/10{ans=ans*10+c;}else{panic!(concat!("overflow when reading ",stringify!($ty)))}}else{ans=ans*10+c;}}ans}}}};($($ty:ty),*)=>{$(sint_readable_impl!($ty);)*};}macro_rules!float_readable_impl{($ty:ty)=>{impl Readable for$ty{type Output=$ty;fn read_from<C:ReadExtConfig>(r:&mut impl ReadExt<C>)->Self::Output{let mut buf=[0;128];let mut buf_ind=0;r.skip_while(u8::is_ascii_whitespace);while let Some(c)=r.next_byte(){if c.is_ascii_whitespace(){break;}buf[buf_ind]=c;buf_ind+=1;}<$ty>::from_str(unsafe{std::str::from_utf8_unchecked(&buf[..buf_ind])}).expect(concat!("error parsing ",stringify!($ty)))}}};($($ty:ty),*)=>{$(float_readable_impl!($ty);)*};}uint_readable_impl!(u8,u16,u32,u64,u128,usize);sint_readable_impl!(i8,i16,i32,i64,i128,isize);float_readable_impl!(f32,f64);impl<T:Readable,const N:usize>Readable for[T;N]{type Output=[T::Output;N];fn read_from<C:ReadExtConfig>(r:&mut impl ReadExt<C>)->Self::Output{let mut arr=MaybeUninit::uninit_array::<N>();for i in&mut arr{i.write(r.read::<T>());}unsafe{MaybeUninit::array_assume_init(arr)}}}pub struct StringLine;impl Readable for StringLine{type Output=Vec<u8>;fn read_from<C:ReadExtConfig>(r:&mut impl ReadExt<C>)->Self::Output{r.peek_byte().expect("eof");let mut s=vec![];while let Some(c)=r.next_byte(){if c==b'\n'{break;}s.push(c);}if let Some(&b'\r')=s.last(){s.pop();}s}}macro_rules!tuple_readable_impl{()=>{};(@impl$($t:ident)*)=>{impl<$($t:Readable),*>Readable for($($t),*,){type Output=($($t::Output),*,);fn read_from<C:ReadExtConfig>(r:&mut impl ReadExt<C>)->Self::Output{($(r.read::<$t>()),*,)}}};($head:ident$($tail:tt)*)=>{tuple_readable_impl!(@impl$head$($tail)*);tuple_readable_impl!($($tail)*);};}tuple_readable_impl!(T1 T2 T3 T4 T5 T6 T7 T8);}pub mod nightly_ext{#![doc=" Copies of nightly stuff"]use std::mem::MaybeUninit;pub mod prelude{pub use super::*;}pub trait MaybeUninitExt<T>{#[allow(clippy::missing_safety_doc)]unsafe fn array_assume_init<const N:usize>(array:[MaybeUninit<T>;N])->[T;N];fn uninit_array<const N:usize>()->[MaybeUninit<T>;N];}impl<T>MaybeUninitExt<T>for MaybeUninit<T>{unsafe fn array_assume_init<const N:usize>(array:[MaybeUninit<T>;N])->[T;N]{let ret=unsafe{(&array as*const _ as*const[T;N]).read()};std::mem::forget(array);ret}fn uninit_array<const N:usize>()->[MaybeUninit<T>;N]{unsafe{MaybeUninit::<[MaybeUninit<T>;N]>::uninit().assume_init()}}}}pub mod solver{#![doc=" Traits defining the interface which a solution interacts with."]use crate::__cargo_equip::crates::lib::io::{self,ReadExt,ReadExtConfig,WriteExt};pub mod prelude{pub use super::{ProblemTestcaseType,StandardSolver};}#[derive(Debug,Copy,Clone,Eq,PartialEq)]pub enum ProblemTestcaseType{SingleTest,MultiTest,MultiTestUntilEof,}pub trait StandardSolver:Default{fn solve<C:ReadExtConfig>(self,inp:&mut impl ReadExt<C>,out:&mut impl WriteExt);fn solve_tc<C:ReadExtConfig>(self,#[allow(unused)]tc_id:u32,inp:&mut impl ReadExt<C>,out:&mut impl WriteExt,){self.solve(inp,out);}}pub fn main<S:StandardSolver,C:ReadExtConfig>(tc_type:ProblemTestcaseType){let mut inp=io::stdin_with_config::<C>();let mut out=io::stdout();match tc_type{ProblemTestcaseType::SingleTest=>{S::default().solve(&mut inp,&mut out);}ProblemTestcaseType::MultiTest=>{let tcs=inp.read::<u32>();for tc in 1..=tcs{S::default().solve_tc(tc,&mut inp,&mut out);}assert!(inp.eof(),"multitest didn't read all input");}ProblemTestcaseType::MultiTestUntilEof=>{for tc in 1..{if inp.eof(){break;}S::default().solve_tc(tc,&mut inp,&mut out);}}}}}pub mod prelude{pub use super::{io::{self,prelude::*},nightly_ext::prelude::*,solver::{self,prelude::*},};}}
    }

    pub(crate) mod macros {
        pub mod lib {}
    }

    pub(crate) mod prelude {pub use crate::__cargo_equip::crates::*;}

    mod preludes {
        pub mod lib {}
    }
}