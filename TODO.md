TODO
---

- ドキュメント関連
	- [ ] それぞれの関数にドキュメンテーションコメントを付ける
	- [ ] README.md の説明書きを加える
- `numerics`
	- [x] `numerics` を分割して、ディレクトリ構造にする
	- [ ] 特殊関数に関する実装を追加する
		- ガンマ、ゼータとか
	- [ ] 任意精度の実数や整数の型を作るか、既存のライブラリの型をインテグレートする
	- `power` 関数
		- [x] `power` の戻り値の型を `SupportsPowerOf` トレイトの型パラメータにするのではなく、 `type PowerOutput;` で指定できるようにする
		- [ ] `power(f64,f32)` や `power(Complex<f64>,f32)`, `power(Complex<f64>,Complex<f32>)` に対応させる
		- [ ] `NonZero**` への対応
			- 2重3重の `from` を使って
		- [ ] `try_from` になっている型への対応
			- `Option<T>` 型として返す
	- [ ] 2次方程式、3次方程式、4次方程式のソルバを実装する
- マクロ関連
	- `new_structure!` マクロ
		- [ ] struct の値として指定できる既存の enum 型について `x:EnumType = EnumType::Var` と通常は表記しているものを `x:EnumType = ::Var` と省略して表記できるようにする
		- [x] カプセル化しただけの struct に対応する
			- `enum` バリアントの named, unnamed で対応させるか
		- [ ] デバッグ出力時にブロック単位で改行されるようにする
		- [ ] それ自体では特に意味のないブロックに対応する
			- `#[pub_all]` とかの属性をまとめて指定できるように
		- [ ] `mod` に対応する
			- マクロ展開された時点で適用されるモジュールを用意するため
	- `par_for_each!` マクロ
		- [ ] NDArray 向けの `each_nd` の他に一般のイテレータ向けの `each` 関数も実装する
		- [ ] NDArray のインデクスからインデクスに関するイテレータを生成できるようにする
		- [ ] `feature="parallel"` が指定されていない限り `par_for_each` を無効にする
	- [ ] トークン系のマクロで、トークンをビルド時に標準エラー出力に出力されるようにしたものを用意
	- [ ] マクロ展開を便利にする手続き型マクロ
		- [ ] 積の形にマクロ展開する `macro_product!` を実装
		- [ ] リストを逆順に返す `macro_rev!` を実装
		- [ ] リストの項目を1つずつ増やしながら呼び出す `macro_dup!` を実装
- イテレータ関連
	- [ ] 配列型の `Zip` に対して並列版を用意する
	- [x] 並列の `Zip` の `IntoIter` に対して `zip_eq` を用意する
	- [x] 直列、並列ともに `zip_longest` を実装する
	- [ ] `zip_longest` として型のデフォルト値で補完するものを用意する
	- [x] タプルに対する `CartesianProduct` の `DoubleEndedIterator` を用意する
	- [ ] `CartesianProduct` の並列版を用意する
	- [ ] 通常の `CartesianProduct` から double ended な `CartesianProduct` に変換できる `.into_double_ended_iter()` を用意
	- [ ] 配列に対する `CartesianProduct` を用意する
	- [ ] 作ったイテレータに対して `.nth()` や `.nth_back()` を実装する
		- `Zip` に関しては含まれるイテレータに丸投げしたらいい
		- `CartesianProduct` に関してはインデクスからより効率的なアルゴリズムを取り出せそう
	- `ExtendedMap` の新しいイテレータ
		- [ ] フォーマッタを実装したマップを用意する
			- [このあたり](https://docs.rs/itertools/latest/itertools/trait.Itertools.html#method.format) を参考にしよう
		- [ ] `Option<T>` をアイテムに持つイテレータ向けに `and_then`, `or_else`, `unwrap_or_else`, `unwrap_or`, `unwrap_or_default`, `map_or_else`, `map_or` を提供する
		- [ ] `Result<T,E>` をアイテムに持つイテレータ向けに `and_then`, `or_else`, `unwrap_or_else`, `unwrap_or`, `unwrap_or_default`, `map_or_else`, `map_or` を提供する
	- [ ] イテレータに `Clone` トレイトを実装する
	- [x] 直列版に対する `.zip_eq()` や `.zip_longest()` を用意する
	- [ ] `unzip` を用意できればいいかな
	- [x] `Iterator.chain` に対して複数のイテレータをチェーンする関数を用意できればいいな
	- [ ] `permutations` や `combination` のイテレータを用意する
	- [ ] 他にも [ここ](https://docs.rs/itertools/0.10.5/itertools/trait.Itertools.html#method.cartesian_product) にある操作の幾つかに対応させる
	- [ ] `par_for_each!` を実装した `Zip` に対応させる
		- 個数に制限があるので、俊敏に振り分けるようにする
- `ndarray` 関連
	- `lanes` に対応する並列イテレータ
	- 複数の要素の書き換え可能な形での参照が可能な `multi_get_mut` の実装
	- スワップにより内部実装の次元間の並び替えが行える関数
		- これは難しいかな...
- その他の新機能
	- [ ] アーカイブ形式の一般化
		- アーカイブからアイテムを削除する機能とか
	- [ ] 多言語対応

`macro_product!` や `macro_rev!` のサンプル
```rust
macro_product! {
	func = (sin) (cos) (tan)
	types = (f64) (f32) (C<f64>) (C<f32>);
	println!("function {} for {}",func,types);
}
// func, types に括弧内の値が代入されて println!(...) が 12 個生成される。
// 括弧の形式は [] () {} の任意にする
macro_rev! { space_separated: A B C } // C B A
// 最初に区切り方を指定し、その後に値を指定する
```
