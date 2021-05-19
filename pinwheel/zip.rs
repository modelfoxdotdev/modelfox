use futures_signals::signal::Signal;
use pin_project::pin_project;
use std::task::Poll;

pub trait ZipSignalTrait<S1, S2>
where
	S1: Signal,
	S2: Signal,
	S1::Item: Clone,
	S2::Item: Clone,
{
	fn zip(self) -> ZipSignal<S1, S2>;
}

impl<S1, S2> ZipSignalTrait<S1, S2> for (S1, S2)
where
	S1: Signal,
	S2: Signal,
	S1::Item: Clone,
	S2::Item: Clone,
{
	fn zip(self) -> ZipSignal<S1, S2> {
		ZipSignal::new(self.0, self.1)
	}
}

#[pin_project(project = ZipSignalProjection)]
pub struct ZipSignal<S1, S2>
where
	S1: Signal,
	S2: Signal,
	S1::Item: Clone,
	S2::Item: Clone,
{
	#[pin]
	s1: S1,
	#[pin]
	s2: S2,
	t1: Option<S1::Item>,
	t2: Option<S2::Item>,
}

impl<S1, S2> ZipSignal<S1, S2>
where
	S1: Signal,
	S2: Signal,
	S1::Item: Clone,
	S2::Item: Clone,
{
	pub fn new(s1: S1, s2: S2) -> ZipSignal<S1, S2> {
		ZipSignal {
			s1,
			s2,
			t1: None,
			t2: None,
		}
	}
}

impl<S1, S2> Signal for ZipSignal<S1, S2>
where
	S1: Signal,
	S2: Signal,
	S1::Item: Clone,
	S2::Item: Clone,
{
	type Item = (S1::Item, S2::Item);
	fn poll_change(
		self: std::pin::Pin<&mut Self>,
		cx: &mut std::task::Context,
	) -> Poll<Option<Self::Item>> {
		let ZipSignalProjection { s1, s2, t1, t2 } = self.project();
		match (s1.poll_change(cx), s2.poll_change(cx)) {
			(Poll::Pending, Poll::Pending) => Poll::Pending,
			(Poll::Ready(None), _) | (_, Poll::Ready(None)) => Poll::Ready(None),
			(Poll::Ready(Some(new_t1)), Poll::Pending) => {
				if let Some(t2) = t2 {
					*t1 = Some(new_t1.clone());
					Poll::Ready(Some((new_t1, t2.clone())))
				} else {
					Poll::Pending
				}
			}
			(Poll::Pending, Poll::Ready(Some(new_t2))) => {
				if let Some(t1) = t1 {
					*t2 = Some(new_t2.clone());
					Poll::Ready(Some((t1.clone(), new_t2)))
				} else {
					Poll::Pending
				}
			}
			(Poll::Ready(Some(t1)), Poll::Ready(Some(t2))) => Poll::Ready(Some((t1, t2))),
		}
	}
}
