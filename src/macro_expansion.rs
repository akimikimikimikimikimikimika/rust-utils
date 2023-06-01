//! ## `macro_expansion`
//! ここでは、各ソースファイルで定義されたタプルに対するトレイトの実装をまとめて行うマクロを、まとめて呼び出している
//! タプルは要素個数ごとに実装が必要であり、その最大個数をここで一元的に制御している

use super::*;

crate::numerics::basic_operations::min_max::implement!(indices: 1 2 3 4 5 6 7 8 9 10 11 );

#[cfg(feature="numerics")]
crate::numerics::primitive_function_extensions::hypot::implement!(indices: 1 2 3 4 5 );

crate::tuples::tuple_to_array::implement!(indices: 0 1 2 3 4 5 6 7 8 9 10 11 12 );

#[cfg(feature="iterator")]
crate::iterator::zip::for_iters::implement!( I0 T0 0 I1 T1 1 I2 T2 2 I3 T3 3 I4 T4 4 I5 T5 5 I6 T6 6 I7 T7 7 I8 T8 8 I9 T9 9 I10 T10 10 I11 T11 11 );

#[cfg(all(feature="iterator",feature="parallel"))]
crate::iterator::zip::for_parallel_iters::implement!( I0 P0 T0 0 I1 P1 T1 1 I2 P2 T2 2 I3 P3 T3 3 I4 P4 T4 4 I5 P5 T5 5 I6 P6 T6 6 I7 P7 T7 7 I8 P8 T8 8 I9 P9 T9 9 I10 P10 T10 10 I11 P11 T11 11 );

#[cfg(feature="iterator")]
crate::iterator::zip::len_equality::implement!(indices: 0 1 2 3 4 5 6 7 8 9 10 11 );

#[cfg(feature="iterator")]
crate::iterator::product::for_iters_tuple::implement!( I0 T0 0 I1 T1 1 I2 T2 2 I3 T3 3 I4 T4 4 I5 T5 5 I6 T6 6 I7 T7 7 I8 T8 8 I9 T9 9 I10 T10 10 I11 T11 11 );

#[cfg(feature="iterator")]
crate::iterator::product::for_double_ended_iters_tuple::implement!( I0 T0 0 I1 T1 1 I2 T2 2 I3 T3 3 I4 T4 4 I5 T5 5 I6 T6 6 I7 T7 7 I8 T8 8 I9 T9 9 I10 T10 10 I11 T11 11 );

#[cfg(feature="iterator")]
crate::iterator::chain::for_iters_tuple::implement!( I0 0 I1 1 I2 2 I3 3 I4 4 I5 5 I6 6 I7 7 I8 8 I9 9 I10 10 I11 11 );

crate::tuples::options::implement!( T0 0 T1 1 T2 2 T3 3 T4 4 T5 5 T6 6 T7 7 T8 8 T9 9 T10 10 T11 11 );

#[cfg(feature="iterator")]
crate::tuples::array::implement!( T0 0 T1 1 T2 2 T3 3 T4 4 T5 5 T6 6 T7 7 T8 8 T9 9 T10 10 T11 11 );
