use super::{LookupTable, LookupTableFromTypes};
use gotham::{
	hyper::{
		header::{HeaderMap, CONTENT_TYPE},
		StatusCode
	},
	router::{non_match::RouteNonMatch, route::matcher::RouteMatcher},
	state::{FromState, State}
};
use mime::Mime;

/**
A route matcher that checks for the presence of a supported content type.

Usage:

```
# use gotham::{helpers::http::response::create_response, hyper::StatusCode, router::builder::*};
# use gotham_restful::matcher::ContentTypeMatcher;
#
let types = vec![mime::TEXT_HTML, mime::TEXT_PLAIN];
let matcher = ContentTypeMatcher::new(types)
	// optionally accept requests with no content type
	.allow_no_type();

# build_simple_router(|route| {
// use the matcher for your request
route.post("/foo")
     .extend_route_matcher(matcher)
	 .to(|state| {
	 	let res = create_response(&state, StatusCode::OK, mime::TEXT_PLAIN, "Correct Content Type!");
		(state, res)
	});
# });
```
*/
#[derive(Clone)]
pub struct ContentTypeMatcher
{
	types : Vec<Mime>,
	lookup_table : LookupTable,
	allow_no_type : bool
}

impl ContentTypeMatcher
{
	/// Create a new `ContentTypeMatcher` with the given supported types that does not allow requests
	/// that don't include a content-type header.
	pub fn new(types : Vec<Mime>) -> Self
	{
		let lookup_table = LookupTable::from_types(types.iter(), false);
		Self { types, lookup_table, allow_no_type: false }
	}
	
	/// Modify this matcher to allow requests that don't include a content-type header.
	pub fn allow_no_type(mut self) -> Self
	{
		self.allow_no_type = true;
		self
	}
}

#[inline]
fn err() -> RouteNonMatch
{
	RouteNonMatch::new(StatusCode::UNSUPPORTED_MEDIA_TYPE)
}

impl RouteMatcher for ContentTypeMatcher
{
	fn is_match(&self, state : &State) -> Result<(), RouteNonMatch>
	{
		HeaderMap::borrow_from(state).get(CONTENT_TYPE)
			.map(|ty| {
				// parse mime type from the content type header
				let mime : Mime = ty.to_str()
					.map_err(|_| err())?
					.parse()
					.map_err(|_| err())?;
				
				// get mime type candidates from the lookup table
				let essence = mime.essence_str();
				let candidates = self.lookup_table.get(essence).ok_or_else(err)?;
				for i in candidates
				{
					let candidate = &self.types[*i];
					
					// check that the candidates have the same suffix - this is not included in the
					// essence string
					if candidate.suffix() != mime.suffix()
					{
						continue
					}
					
					// check that this candidate has at least the parameters that the content type
					// has and that their values are equal
					if candidate.params().any(|(key, value)| mime.get_param(key) != Some(value))
					{
						continue
					}
					
					// this candidate matches
					return Ok(())
				}
				
				// no candidates found
				Err(err())
			}).unwrap_or_else(|| {
				// no type present
				if self.allow_no_type { Ok(()) } else { Err(err()) }
			})
	}
}


#[cfg(test)]
mod test
{
	use super::*;
	
	fn with_state<F>(content_type : Option<&str>, block : F)
	where F : FnOnce(&mut State) -> ()
	{
		State::with_new(|state| {
			let mut headers = HeaderMap::new();
			if let Some(ty) = content_type
			{
				headers.insert(CONTENT_TYPE, ty.parse().unwrap());
			}
			state.put(headers);
			block(state);
		});
	}
	
	#[test]
	fn empty_type_list()
	{
		let matcher = ContentTypeMatcher::new(Vec::new());
		with_state(None, |state| assert!(matcher.is_match(&state).is_err()));
		with_state(Some("text/plain"), |state| assert!(matcher.is_match(&state).is_err()));
		
		let matcher = matcher.allow_no_type();
		with_state(None, |state| assert!(matcher.is_match(&state).is_ok()));
	}
	
	#[test]
	fn simple_type()
	{
		let matcher = ContentTypeMatcher::new(vec![mime::TEXT_PLAIN]);
		with_state(None, |state| assert!(matcher.is_match(&state).is_err()));
		with_state(Some("text/plain"), |state| assert!(matcher.is_match(&state).is_ok()));
		with_state(Some("text/plain; charset=utf-8"), |state| assert!(matcher.is_match(&state).is_ok()));
	}
	
	#[test]
	fn complex_type()
	{
		let matcher = ContentTypeMatcher::new(vec!["image/svg+xml; charset=utf-8".parse().unwrap()]);
		with_state(Some("image/svg"), |state| assert!(matcher.is_match(&state).is_err()));
		with_state(Some("image/svg+xml"), |state| assert!(matcher.is_match(&state).is_err()));
		with_state(Some("image/svg+xml; charset=utf-8"), |state| assert!(matcher.is_match(&state).is_ok()));
		with_state(Some("image/svg+xml; charset=utf-8; eol=lf"), |state| assert!(matcher.is_match(&state).is_ok()));
		with_state(Some("image/svg+xml; charset=us-ascii"), |state| assert!(matcher.is_match(&state).is_err()));
		with_state(Some("image/svg+json; charset=utf-8"), |state| assert!(matcher.is_match(&state).is_err()));
	}
	
	#[test]
	fn type_mismatch()
	{
		let matcher = ContentTypeMatcher::new(vec![mime::TEXT_HTML]);
		with_state(Some("text/plain"), |state| assert!(matcher.is_match(&state).is_err()));
	}
}
