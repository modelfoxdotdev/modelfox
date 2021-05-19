use quote::quote;

pub fn component_builder(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
	let input: syn::DeriveInput = syn::parse2(input)?;
	let ident = &input.ident;
	let data = match &input.data {
		syn::Data::Struct(data) => data,
		_ => {
			return Err(syn::Error::new_spanned(
				input,
				"this macro can only be used on a struct",
			))
		}
	};
	let mut required_fields = Vec::new();
	let mut optional_fields = Vec::new();
	let mut children_field = None;
	for field in data.fields.iter() {
		let field_ident = &field.ident;
		let field_ty = &field.ty;
		enum Kind {
			Required,
			Optional,
			Children,
		}
		let kind = if field
			.attrs
			.iter()
			.any(|attr| attr.path.is_ident("optional"))
		{
			Kind::Optional
		} else if field
			.attrs
			.iter()
			.any(|attr| attr.path.is_ident("children"))
		{
			Kind::Children
		} else {
			Kind::Required
		};
		match kind {
			Kind::Required => {
				required_fields.push((field_ident, field_ty));
			}
			Kind::Optional => {
				optional_fields.push((field_ident, field_ty));
			}
			Kind::Children => {
				children_field = Some((field_ident, field_ty));
			}
		}
	}
	let new_args = required_fields.iter().map(|(field_ident, field_ty)| {
		quote! {
			#field_ident: impl Into<#field_ty>
		}
	});
	let new_required_fields = required_fields.iter().map(|(field_ident, _)| {
		quote! {
			#field_ident: #field_ident.into()
		}
	});
	let new_optional_fields = optional_fields.iter().map(|(field_ident, _)| {
		quote! {
			#field_ident: Default::default()
		}
	});
	let new_children_field = children_field.map(|_| {
		quote! {
			children: Vec::new()
		}
	});
	let optional_fns = optional_fields.iter().map(|(field_ident, field_ty)| {
		quote! {
			pub fn #field_ident(mut self, #field_ident: impl Into<#field_ty>) -> #ident {
				self.#field_ident = #field_ident.into();
				self
			}
		}
	});
	let child_fn = children_field.map(|_| {
		quote! {
			pub fn child<T>(mut self, child: T) -> #ident where T: Into<Node> {
				let child = child.into();
				self.children.push(child);
				self
			}
		}
	});
	let children_fn = children_field.map(|_| {
		quote! {
			pub fn children<T, I>(mut self, children: I) -> #ident where T: Into<Node>, I: IntoIterator<Item = T> {
				for child in children {
					let child = child.into();
					self.children.push(child);
				}
				self
			}
		}
	});
	let child_signal_fn = children_field.map(|_| {
		quote! {
			pub fn child_signal<T, S>(mut self, signal: S) -> Self
			where
				T: Into<Node>,
				S: 'static + Unpin + Signal<Item = T>,
			{
				self.children.push(Node::Signal(SignalNode::new(signal)));
				self
			}
		}
	});
	let child_signal_vec_fn = children_field.map(|_| {
		quote! {
			pub fn child_signal_vec<T, S>(mut self, signal_vec: S) -> Self
			where
				T: Into<Node>,
				S: 'static + Unpin + SignalVec<Item = T>,
			{
				self.children.push(Node::SignalVec(SignalVecNode::new(signal_vec)));
				self
			}
		}
	});
	Ok(quote! {
		impl #ident {
			pub fn new(#(#new_args),*) -> #ident {
				#ident {
					#(#new_required_fields,)*
					#(#new_optional_fields,)*
					#new_children_field
				}
			}
			#(#optional_fns)*
			#child_fn
			#children_fn
			#child_signal_fn
			#child_signal_vec_fn
		}
	})
}
