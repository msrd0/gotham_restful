use proc_macro2::{Delimiter, TokenStream, TokenTree};
use std::iter;
use syn::{Error, Lit, LitBool, LitStr, Result};

pub(crate) trait CollectToResult {
	type Item;

	fn collect_to_result(self) -> Result<Vec<Self::Item>>;
}

impl<Item, I> CollectToResult for I
where
	I: Iterator<Item = Result<Item>>
{
	type Item = Item;

	fn collect_to_result(self) -> Result<Vec<Item>> {
		self.fold(Ok(Vec::new()), |res, code| match (code, res) {
			(Ok(code), Ok(mut codes)) => {
				codes.push(code);
				Ok(codes)
			},
			(Ok(_), Err(errors)) => Err(errors),
			(Err(err), Ok(_)) => Err(err),
			(Err(err), Err(mut errors)) => {
				errors.combine(err);
				Err(errors)
			}
		})
	}
}

pub(crate) trait ExpectLit {
	fn expect_bool(self) -> Result<LitBool>;
	fn expect_str(self) -> Result<LitStr>;
}

impl ExpectLit for Lit {
	fn expect_bool(self) -> Result<LitBool> {
		match self {
			Self::Bool(bool) => Ok(bool),
			_ => Err(Error::new(self.span(), "Expected boolean literal"))
		}
	}

	fn expect_str(self) -> Result<LitStr> {
		match self {
			Self::Str(str) => Ok(str),
			_ => Err(Error::new(self.span(), "Expected string literal"))
		}
	}
}

pub(crate) fn remove_parens(input: TokenStream) -> TokenStream {
	let iter = input.into_iter().flat_map(|tt| {
		if let TokenTree::Group(group) = &tt {
			if group.delimiter() == Delimiter::Parenthesis {
				return Box::new(group.stream().into_iter()) as Box<dyn Iterator<Item = TokenTree>>;
			}
		}
		Box::new(iter::once(tt))
	});
	iter.collect()
}
