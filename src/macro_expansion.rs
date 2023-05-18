//! ## `macro_expansion`
//! ここでは、各ソースファイルで定義されたタプルに対するトレイトの実装をまとめて行うマクロを、まとめて呼び出している
//! タプルは要素個数ごとに実装が必要であり、その最大個数をここで一元的に制御している

use super::*;

impl_min_max!(indices: 1 2 3 4 5 6 7 8 9 10 11 );

#[cfg(feature="numerics")]
impl_hypot!(indices: 1 2 3 4 5 );

impl_tuple_to_array!(indices: 0 1 2 3 4 5 6 7 8 9 10 11 12 );

impl_zipped_iter!( T0 0 T1 1 T2 2 T3 3 T4 4 T5 5 T6 6 T7 7 T8 8 T9 9 T10 10 T11 11 );

impl_chain_iter!( I0 0 I1 1 I2 2 I3 3 I4 4 I5 5 I6 6 I7 7 I8 8 I9 9 I10 10 I11 11 );

impl_zip_options!( T0 0 T1 1 T2 2 T3 3 T4 4 T5 5 T6 6 T7 7 T8 8 T9 9 T10 10 T11 11 );

impl_zip_arrays!( T0 0 T1 1 T2 2 T3 3 T4 4 T5 5 T6 6 T7 7 T8 8 T9 9 T10 10 T11 11 );
