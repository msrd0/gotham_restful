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
