use super::*;
use std::cmp;

pub trait IteratorExtension<I> where I: Iterator {

	fn cycle_n(self,repeat:usize) -> CycleN<Self>
		where Self: Clone + Sized;

	fn min_max(self) -> Option<(I::Item,I::Item)>
		where I: Sized, I::Item: Clone + Ord;
	fn min_max_by<F>(self,compare:F) -> Option<(I::Item,I::Item)> where
		I: Sized, I::Item: Clone + Ord,
		F: FnMut(&I::Item,&I::Item) -> cmp::Ordering;

}

impl<I> IteratorExtension<I> for I where I: Iterator {

	fn cycle_n(self,repeat:usize) -> CycleN<Self>
		where Self: Clone + Sized
	{
		CycleN { iterator: self.clone(), original: self, whole_count: repeat, current_count: repeat }
	}



	fn min_max(self) -> Option<(I::Item,I::Item)>
		where I: Sized, I::Item: Clone + Ord
	{
		self.min_max_by(Ord::cmp)
	}

	fn min_max_by<F>(mut self,mut compare:F) -> Option<(I::Item,I::Item)> where
		I: Sized, I::Item: Clone + Ord,
		F: FnMut(&I::Item,&I::Item) -> cmp::Ordering
	{
		let first = self.next()?;
		Some( self.fold(
			(first.clone(),first),
			move |(min_val,max_val),item| {
				(
					cmp::min_by(min_val,item.clone(),&mut compare),
					cmp::max_by(max_val,item,&mut compare)
				)
			}
		) )
	}

}



#[derive(Clone)]
pub struct CycleN<I> {
	original: I,
	iterator: I,
	whole_count: usize,
	current_count: usize
}

impl<I> Iterator for CycleN<I> where I: Clone + Iterator {

	type Item = I::Item;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		match (self.iterator.next(),self.current_count) {
			(_,0) => None,
			(None,1) => None,
			(None,_) => {
				self.current_count -= 1;
				self.iterator = self.original.clone();
				self.iterator.next()
			},
			(s,_) => s
		}
	}

	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>) {
		match (self.original.size_hint(),self.whole_count) {
			((0,Some(0)),_)|(_,0) => (0,Some(0)),
			((l,u),n) => (
				l.checked_mul(n).unwrap_or(usize::MAX),
				u.and_then(|u| u.checked_mul(n) )
			)
		}
	}

}



pub struct Zip3<A,B,C> {
	a: A, b: B, c: C
}

impl<A,B,C> Iterator for Zip3<A,B,C> where A: Iterator, B: Iterator, C: Iterator {
	type Item = (A::Item,B::Item,C::Item);
	fn next(&mut self) -> Option<Self::Item> {
		match (self.a.next(),self.b.next(),self.c.next()) {
			(Some(a),Some(b),Some(c)) => Some((a,b,c)),
			_ => None
		}
	}
	fn size_hint(&self) -> (usize, Option<usize>) {
		let (a,b,c) = (self.a.size_hint(),self.b.size_hint(),self.c.size_hint());
		let l = [a.0,b.0,c.0].minimum();
		let u = [a.1,b.1,c.1].iter().filter_map(|x| x.as_ref()).min().map(|x| *x);
		(l,u)
	}
}
