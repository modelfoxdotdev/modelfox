#[macro_export]
macro_rules! pzip {
	($($e:expr),* $(,)*) => {
		rayon::iter::IntoParallelIterator::into_par_iter(($($e,)*))
	};
}
