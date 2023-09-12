use proc_macro2::Ident;
use syn::{spanned::Spanned as _, Error, Expr, Lit, LitBool, LitStr, Result};

pub(crate) trait CollectToResult {
	type Item;

	fn collect_to_result(self) -> Result<Vec<Self::Item>>;
}

impl<Item, I> CollectToResult for I
where
	I: Iterator<Item = Result<Item>>
{
	type Item = Item;

	#[allow(clippy::manual_try_fold)] // false positive
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

pub(crate) trait IntoIdent {
	fn into_ident(self) -> Ident;
}

impl IntoIdent for LitStr {
	fn into_ident(self) -> Ident {
		Ident::new(&self.value(), self.span())
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

fn expect_lit(expr: Expr) -> syn::Result<Lit> {
	match expr {
		Expr::Lit(lit) => Ok(lit.lit),
		_ => Err(syn::Error::new(expr.span(), "Expected literal"))
	}
}

impl ExpectLit for Expr {
	fn expect_bool(self) -> syn::Result<LitBool> {
		expect_lit(self)?.expect_bool()
	}

	fn expect_str(self) -> syn::Result<LitStr> {
		expect_lit(self)?.expect_str()
	}
}
