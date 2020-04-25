use super::{LookupTable, LookupTableFromTypes};
use gotham::{
	hyper::{
		header::{HeaderMap, ACCEPT},
		StatusCode
	},
	router::{non_match::RouteNonMatch, route::matcher::RouteMatcher},
	state::{FromState, State}
};
use mime::Mime;
use std::{
	num::ParseFloatError,
	str::FromStr
};
use thiserror::Error;


/// A mime type that is optionally weighted with a quality.
#[derive(Debug)]
struct QMime
{
	mime : Mime,
	weight : Option<f32>
}

impl QMime
{
	fn new(mime : Mime, weight : Option<f32>) -> Self
	{
		Self { mime, weight }
	}
}

#[derive(Debug, Error)]
enum QMimeError
{
	#[error("Unable to parse mime type: {0}")]
	MimeError(#[from] mime::FromStrError),
	#[error("Unable to parse mime quality: {0}")]
	NumError(#[from] ParseFloatError)
}

impl FromStr for QMime
{
	type Err = QMimeError;
	
	fn from_str(str : &str) -> Result<Self, Self::Err>
	{
		match str.find(";q=") {
			None => Ok(Self::new(str.parse()?, None)),
			Some(index) => {
				let mime = str[..index].parse()?;
				let weight = str[index+3..].parse()?;
				Ok(Self::new(mime, Some(weight)))
			}
		}
	}
}


/**
A route matcher that checks whether the supported types match the accept header of the request.

Usage:

```
# use gotham::{helpers::http::response::create_response, hyper::StatusCode, router::builder::*};
# use gotham_restful::matcher::AcceptHeaderMatcher;
#
# const img_content : &[u8] = b"This is the content of a webp image";
#
# let IMAGE_WEBP : mime::Mime = "image/webp".parse().unwrap();
let types = vec![IMAGE_WEBP];
let matcher = AcceptHeaderMatcher::new(types);

# build_simple_router(|route| {
// use the matcher for your request
route.post("/foo")
     .extend_route_matcher(matcher)
	 .to(|state| {
	 	// we know that the client is a modern browser and can handle webp images
# let IMAGE_WEBP : mime::Mime = "image/webp".parse().unwrap();
	 	let res = create_response(&state, StatusCode::OK, IMAGE_WEBP, img_content);
		(state, res)
	});
# });
```
*/
#[derive(Clone)]
pub struct AcceptHeaderMatcher
{
	types : Vec<Mime>,
	lookup_table : LookupTable
}

impl AcceptHeaderMatcher
{
	/// Create a new `AcceptHeaderMatcher` with the given types that can be produced by the route.
	pub fn new(types : Vec<Mime>) -> Self
	{
		let lookup_table = LookupTable::from_types(types.iter(), true);
		Self { types, lookup_table }
	}
}

#[inline]
fn err() -> RouteNonMatch
{
	RouteNonMatch::new(StatusCode::NOT_ACCEPTABLE)
}

impl RouteMatcher for AcceptHeaderMatcher
{
	fn is_match(&self, state : &State) -> Result<(), RouteNonMatch>
	{
		HeaderMap::borrow_from(state).get(ACCEPT)
			.map(|header| {
				// parse mime types from the accept header
				let acceptable = header.to_str()
					.map_err(|_| err())?
					.split(',')
					.map(|str| str.trim().parse())
					.collect::<Result<Vec<QMime>, _>>()
					.map_err(|_| err())?;
				
				for qmime in acceptable
				{
					// get mime type candidates from the lookup table
					let essence = qmime.mime.essence_str();
					let candidates = match self.lookup_table.get(essence) {
						Some(candidates) => candidates,
						None => continue
					};
					for i in candidates
					{
						let candidate = &self.types[*i];
						
						// check that the candidates have the same suffix - this is not included in the
						// essence string
						if candidate.suffix() != qmime.mime.suffix()
						{
							continue
						}
						
						// this candidate matches - params don't play a role in accept header matching
						return Ok(())
					}
				}
				
				// no candidates found
				Err(err())
			}).unwrap_or_else(|| {
				// no accept header - assume all types are acceptable
				Ok(())
			})
	}
}


#[cfg(test)]
mod test
{
	use super::*;
	
	fn with_state<F>(accept : Option<&str>, block : F)
	where F : FnOnce(&mut State) -> ()
	{
		State::with_new(|state| {
			let mut headers = HeaderMap::new();
			if let Some(acc) = accept
			{
				headers.insert(ACCEPT, acc.parse().unwrap());
			}
			state.put(headers);
			block(state);
		});
	}
	
	#[test]
	fn no_accept_header()
	{
		let matcher = AcceptHeaderMatcher::new(vec!(mime::TEXT_PLAIN));
		with_state(None, |state| assert!(matcher.is_match(&state).is_ok()));
	}
	
	#[test]
	fn single_mime_type()
	{
		let matcher = AcceptHeaderMatcher::new(vec!(mime::TEXT_PLAIN, mime::IMAGE_PNG));
		with_state(Some("text/plain"), |state| assert!(matcher.is_match(&state).is_ok()));
		with_state(Some("text/html"), |state| assert!(matcher.is_match(&state).is_err()));
		with_state(Some("image/png"), |state| assert!(matcher.is_match(&state).is_ok()));
		with_state(Some("image/webp"), |state| assert!(matcher.is_match(&state).is_err()));
	}
	
	#[test]
	fn star_star()
	{
		let matcher = AcceptHeaderMatcher::new(vec!(mime::IMAGE_PNG));
		with_state(Some("*/*"), |state| assert!(matcher.is_match(&state).is_ok()));
	}
	
	#[test]
	fn image_star()
	{
		let matcher = AcceptHeaderMatcher::new(vec!(mime::IMAGE_PNG));
		with_state(Some("image/*"), |state| assert!(matcher.is_match(&state).is_ok()));
	}
	
	#[test]
	fn complex_header()
	{
		let matcher = AcceptHeaderMatcher::new(vec!(mime::IMAGE_PNG));
		with_state(Some("text/html,image/webp;q=0.8"), |state| assert!(matcher.is_match(&state).is_err()));
		with_state(Some("text/html,image/webp;q=0.8,*/*;q=0.1"), |state| assert!(matcher.is_match(&state).is_ok()));
	}
}
