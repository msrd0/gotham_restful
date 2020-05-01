use proc_macro2::{
	Delimiter,
	TokenStream as TokenStream2,
	TokenTree
};
use std::iter;
use syn::Error;

pub trait CollectToResult
{
	type Item;
	
	fn collect_to_result(self) -> Result<Vec<Self::Item>, Error>;
}

impl<Item, I> CollectToResult for I
where
	I : Iterator<Item = Result<Item, Error>>
{
	type Item = Item;
	
	fn collect_to_result(self) -> Result<Vec<Item>, Error>
	{
		self.fold(<Result<Vec<Item>, Error>>::Ok(Vec::new()), |res, code| {
		    match (code, res) {
		        (Ok(code), Ok(mut codes)) => { codes.push(code); Ok(codes) },
		        (Ok(_), Err(errors)) => Err(errors),
		        (Err(err), Ok(_)) => Err(err),
		        (Err(err), Err(mut errors)) => { errors.combine(err); Err(errors) }
		    }
		})
	}
}


pub fn remove_parens(input : TokenStream2) -> TokenStream2
{
	let iter = input.into_iter().flat_map(|tt| {
		if let TokenTree::Group(group) = &tt
		{
			if group.delimiter() == Delimiter::Parenthesis
			{
				return Box::new(group.stream().into_iter()) as Box<dyn Iterator<Item = TokenTree>>;
			}
		}
		Box::new(iter::once(tt))
	});
	iter.collect()
}
