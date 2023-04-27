TODO
---

- `numerics`
	- [x] `numerics` を分割して、ディレクトリ構造にする
	- [ ] 特殊関数に関する実装を追加する
		- ガンマ、ゼータとか
	- [ ] 任意精度の実数や整数の型を作るか、既存のライブラリの型をインテグレートする
- `new_structure!` マクロ
	- [ ] `Data` を `enum` 型にする代わりに、ジェネリクスにより `impl Data` として `Struct` や `Enum` を `Data` に当てはめる
		- これをする意味は本当にあるのでしょうか
	- [ ] struct の値として指定できる既存の enum 型について `x:EnumType = EnumType::Var` と通常は表記しているものを `x:EnumType = ::Var` と省略して表記できるようにする
	- [ ] カプセル化しただけの struct に対応する
		- `enum` バリアントの named, unnamed で対応させるか
	- [ ] デバッグ出力時にブロック単位で改行されるようにする
	- [ ] それ自体では特に意味のないブロックに対応する
		- `#[pub_all]` とかの属性をまとめて指定できるように
	- [ ] `mod` に対応する
		- マクロ展開された時点で適用されるモジュールを用意するため
- `par_for_each!` マクロ
	- [ ] NDArray 向けの `each_nd` の他に一般のイテレータ向けの `each` 関数も実装する
- [ ] トークン系のマクロで、トークンをビルド時に標準エラー出力に出力されるようにしたものを用意
- [x] `Zip3` が現在用意されているが、これを複数個の Zip にも対応させる
	- 同時に `unzip` も用意できればいいかな
- `par_for_each!` を `ParallelIterator` の多数 Zip に対応する
	- `(IndexedParallelIterator,..).into_par_iter()` により生成できる `MultiZip` 型
	- `MultiZip` は個数に制限があるので、そこを俊敏に振り分けるようにする
	- `itertools` の `multi_cartesian_product` みたいに `IntoIterator` を返すイテレータからベクターを生成したい (以下イテレータ型と呼び、 `MultiZip` の方式をタプル型と呼ぶ)
- `itertools` にあるイテレータを真似して `ParallelIterator` に対応させる
	- `product` とか `permutations` とか
	- `multi_cartesian_product` に対してはイテレータ式とタプル式の両方に対応させたい
	- [ここ](https://docs.rs/itertools/0.10.5/itertools/trait.Itertools.html#method.cartesian_product) にある操作の幾つかに対応させたい
- [x] 同じ型のタプル `(T,T,...)` を配列 `[T;N]` に変換するトレイト
	- どれだけの個数を用意すれば良いだろうか
- `power` 関数
	- [ ] `power(f64,f32)` や `power(Complex<f64>,f32)`, `power(Complex<f64>,Complex<f32>)` に対応させる
	- [ ] `NonZero**` への対応
		- 2重3重の `from` を使って
	- [ ] `try_from` になっている型への対応
		- `Option<T>` 型として返す
- [ ] 積の形にマクロ展開する手続き型マクロ `macro_product!` を実装
```rust
macro_product! {
	func = (sin) (cos) (tan)
	types = (f64) (f32) (C<f64>) (C<f32>);
	println!("function {} for {}",func,types);
}
// func, types に括弧内の値が代入されて println!(...) が 12 個生成される。
// 括弧の形式は [] () {} の任意にする
```
- [x] `smart_for_each!` の移動
- [ ] アーカイブ形式の一般化
	- アーカイブからアイテムを削除する機能とか
- [ ] 多言語対応
- [ ] ドキュメントを拡充