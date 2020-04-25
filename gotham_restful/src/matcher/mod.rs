use itertools::Itertools;
use mime::Mime;
use std::collections::HashMap;

mod accept;
pub use accept::AcceptHeaderMatcher;

mod content_type;
pub use content_type::ContentTypeMatcher;

type LookupTable = HashMap<String, Vec<usize>>;

trait LookupTableFromTypes
{
	fn from_types<'a, I : Iterator<Item = &'a Mime>>(types : I, include_stars : bool) -> Self;
}

impl LookupTableFromTypes for LookupTable
{
	fn from_types<'a, I : Iterator<Item = &'a Mime>>(types : I, include_stars : bool) -> Self
	{
		if include_stars
		{
			types
				.enumerate()
				.flat_map(|(i, mime)| vec![("*/*".to_owned(), i), (format!("{}/*", mime.type_()), i), (mime.essence_str().to_owned(), i)].into_iter())
				.into_group_map()
		}
		else
		{
			types
				.enumerate()
				.map(|(i, mime)| (mime.essence_str().to_owned(), i))
				.into_group_map()
		}
	}
}
