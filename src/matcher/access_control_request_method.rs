use gotham::{
	hyper::{header::{ACCESS_CONTROL_REQUEST_METHOD, HeaderMap}, Method, StatusCode},
	router::{non_match::RouteNonMatch, route::matcher::RouteMatcher},
	state::{FromState, State}
};

/// A route matcher that checks whether the value of the `Access-Control-Request-Method` header matches the defined value.
/// 
/// Usage:
/// 
/// ```rust
/// # use gotham::{helpers::http::response::create_empty_response,
/// #   hyper::{header::ACCESS_CONTROL_ALLOW_METHODS, Method, StatusCode},
/// #   router::builder::*
/// # };
/// # use gotham_restful::matcher::AccessControlRequestMethodMatcher;
/// let matcher = AccessControlRequestMethodMatcher::new(Method::PUT);
/// 
/// # build_simple_router(|route| {
/// // use the matcher for your request
/// route.options("/foo")
/// 	.extend_route_matcher(matcher)
/// 	.to(|state| {
/// 		// we know that this is a CORS preflight for a PUT request
/// 		let mut res = create_empty_response(&state, StatusCode::NO_CONTENT);
/// 		res.headers_mut().insert(ACCESS_CONTROL_ALLOW_METHODS, "PUT".parse().unwrap());
/// 		(state, res)
/// 	});
/// # });
/// ```
#[derive(Clone, Debug)]
pub struct AccessControlRequestMethodMatcher
{
	method : Method
}

impl AccessControlRequestMethodMatcher
{
	pub fn new(method : Method) -> Self
	{
		Self { method }
	}
}

impl RouteMatcher for AccessControlRequestMethodMatcher
{
	fn is_match(&self, state : &State) -> Result<(), RouteNonMatch>
	{
		match HeaderMap::borrow_from(state).get(ACCESS_CONTROL_REQUEST_METHOD)
			.and_then(|value| value.to_str().ok())
			.and_then(|str| str.parse::<Method>().ok())
		{
			Some(m) if m == self.method => Ok(()),
			_ => Err(RouteNonMatch::new(StatusCode::NOT_FOUND))
		}
	}
}
