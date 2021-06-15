use crate::prelude::*;
use std::borrow::Cow;

pub struct Dehydrate<T>
where
	T: Component + serde::Serialize + serde::de::DeserializeOwned,
{
	id: Cow<'static, str>,
	component: T,
}

impl<T> Dehydrate<T>
where
	T: Component + serde::Serialize + serde::de::DeserializeOwned,
{
	pub fn new(id: impl Into<Cow<'static, str>>, component: T) -> Dehydrate<T> {
		Dehydrate {
			id: id.into(),
			component,
		}
	}
}

impl<T> Component for Dehydrate<T>
where
	T: Component + serde::Serialize + serde::de::DeserializeOwned,
{
	fn into_node(self) -> Node {
		div()
			.id(self.id)
			.attribute(
				"data-component",
				serde_json::to_string(&self.component).unwrap(),
			)
			.child(self.component)
			.into_node()
	}
}
