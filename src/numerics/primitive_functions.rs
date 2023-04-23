use super::*;


pub fn exp<F:Float>(x:F) -> F {
	return x.exp();
}

pub fn log<F:Float>(x:F) -> F {
	return x.ln();
}

pub fn sin<F:Float>(x:F) -> F {
	return x.sin();
}

pub fn cos<F:Float>(x:F) -> F {
	return x.cos();
}

pub fn tan<F:Float>(x:F) -> F {
	return x.tan();
}

pub fn sinh<F:Float>(x:F) -> F {
	return x.sinh();
}

pub fn cosh<F:Float>(x:F) -> F {
	return x.cosh();
}

pub fn tanh<F:Float>(x:F) -> F {
	return x.tanh();
}

pub fn atan2<F:Float>(y:F,x:F) -> F {
	return y.atan2(x);
}



/// `hypot` 関数を多数の要素でも使えるようにする
mod hypot {
	use super::*;

	compose_struct! {
		pub trait Iter<T> = IntoIterator<Item=T>;
	}

	pub fn hypot<F:Float>(x:F,y:F) -> F {
		return y.hypot(x);
	}

	pub trait HypotFn<T> {
		fn hypot(self) -> T;
	}
	impl<T:Float, I:Iter<T>> HypotFn<T> for I {
		fn hypot(self) -> T {
			self.into_iter()
			.reduce( |a,v| a.hypot(v) )
			.unwrap_or(T::zero())
		}
	}

}
pub use hypot::*;
