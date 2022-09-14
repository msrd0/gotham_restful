use gotham::{
	handler::HandlerFuture,
	helpers::http::response::create_empty_response,
	hyper::{
		header::{
			HeaderMap, HeaderName, HeaderValue, ACCESS_CONTROL_ALLOW_CREDENTIALS,
			ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS,
			ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_MAX_AGE, ACCESS_CONTROL_REQUEST_HEADERS,
			ACCESS_CONTROL_REQUEST_METHOD, ORIGIN, VARY
		},
		Body, Method, Response, StatusCode
	},
	middleware::Middleware,
	pipeline::PipelineHandleChain,
	prelude::*,
	router::{builder::ExtendRouteMatcher, route::matcher::AccessControlRequestMethodMatcher},
	state::State
};
use std::{panic::RefUnwindSafe, pin::Pin};

/// Specify the allowed origins of the request. It is up to the browser to check the validity of the
/// origin. This, when sent to the browser, will indicate whether or not the request's origin was
/// allowed to make the request.
#[derive(Clone, Debug)]
pub enum Origin {
	/// Do not send any `Access-Control-Allow-Origin` headers.
	None,
	/// Send `Access-Control-Allow-Origin: *`. Note that browser will not send credentials.
	Star,
	/// Set the `Access-Control-Allow-Origin` header to a single origin.
	Single(String),
	/// Copy the `Origin` header into the `Access-Control-Allow-Origin` header.
	Copy
}

impl Default for Origin {
	fn default() -> Self {
		Self::None
	}
}

impl Origin {
	/// Get the header value for the `Access-Control-Allow-Origin` header.
	fn header_value(&self, state: &State) -> Option<HeaderValue> {
		match self {
			Self::None => None,
			Self::Star => Some("*".parse().unwrap()),
			Self::Single(origin) => Some(origin.parse().unwrap()),
			Self::Copy => {
				let headers = HeaderMap::borrow_from(state);
				headers.get(ORIGIN).map(Clone::clone)
			}
		}
	}

	/// Returns true if the `Vary` header has to include `Origin`.
	fn varies(&self) -> bool {
		matches!(self, Self::Copy)
	}
}

/// Specify the allowed headers of the request. It is up to the browser to check that only the allowed
/// headers are sent with the request.
#[derive(Clone, Debug)]
pub enum Headers {
	/// Do not send any `Access-Control-Allow-Headers` headers.
	None,
	/// Set the `Access-Control-Allow-Headers` header to the following header list. If empty, this
	/// is treated as if it was [None].
	List(Vec<HeaderName>),
	/// Copy the `Access-Control-Request-Headers` header into the `Access-Control-Allow-Header`
	/// header.
	Copy
}

impl Default for Headers {
	fn default() -> Self {
		Self::None
	}
}

impl Headers {
	/// Get the header value for the `Access-Control-Allow-Headers` header.
	fn header_value(&self, state: &State) -> Option<HeaderValue> {
		match self {
			Self::None => None,
			Self::List(list) => Some(list.join(",").parse().unwrap()),
			Self::Copy => {
				let headers = HeaderMap::borrow_from(state);
				headers
					.get(ACCESS_CONTROL_REQUEST_HEADERS)
					.map(Clone::clone)
			}
		}
	}

	/// Returns true if the `Vary` header has to include `Origin`.
	fn varies(&self) -> bool {
		matches!(self, Self::Copy)
	}
}

/// This is the configuration that the CORS handler will follow. Its default configuration is basically
/// not to touch any responses, resulting in the browser's default behaviour.
///
/// To change settings, you need to put this type into gotham's [State]:
///
/// ```rust,no_run
/// # use gotham::{router::builder::*, pipeline::*, state::State};
/// # use gotham_restful::{*, cors::Origin};
/// # #[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_doctest_main))]
/// fn main() {
/// 	let cors = CorsConfig {
/// 		origin: Origin::Star,
/// 		..Default::default()
/// 	};
/// 	let (chain, pipelines) = single_pipeline(new_pipeline().add(cors).build());
/// 	gotham::start("127.0.0.1:8080", build_router(chain, pipelines, |route| {
/// your routing logic
/// 	}));
/// }
/// ```
///
/// This easy approach allows you to have one global cors configuration. If you prefer to have separate
/// configurations for different scopes, you need to register the middleware inside your routing logic:
///
/// ```rust,no_run
/// # use gotham::{router::builder::*, pipeline::*, state::State};
/// # use gotham_restful::{*, cors::Origin};
/// let pipelines = new_pipeline_set();
///
/// // The first cors configuration
/// let cors_a = CorsConfig {
/// 	origin: Origin::Star,
/// 	..Default::default()
/// };
/// let (pipelines, chain_a) = pipelines.add(new_pipeline().add(cors_a).build());
///
/// // The second cors configuration
/// let cors_b = CorsConfig {
/// 	origin: Origin::Copy,
/// 	..Default::default()
/// };
/// let (pipelines, chain_b) = pipelines.add(new_pipeline().add(cors_b).build());
///
/// let pipeline_set = finalize_pipeline_set(pipelines);
/// gotham::start(
/// 	"127.0.0.1:8080",
/// 	build_router((), pipeline_set, |route| {
/// 		// routing without any cors config
/// 		route.with_pipeline_chain((chain_a, ()), |route| {
/// 			// routing with cors config a
/// 		});
/// 		route.with_pipeline_chain((chain_b, ()), |route| {
/// 			// routing with cors config b
/// 		});
/// 	})
/// );
/// ```
#[derive(Clone, Debug, Default, NewMiddleware, StateData)]
pub struct CorsConfig {
	/// The allowed origins.
	pub origin: Origin,
	/// The allowed headers.
	pub headers: Headers,
	/// The amount of seconds that the preflight request can be cached.
	pub max_age: u64,
	/// Whether or not the request may be made with supplying credentials.
	pub credentials: bool
}

impl Middleware for CorsConfig {
	fn call<Chain>(self, mut state: State, chain: Chain) -> Pin<Box<HandlerFuture>>
	where
		Chain: FnOnce(State) -> Pin<Box<HandlerFuture>>
	{
		state.put(self);
		chain(state)
	}
}

/// Handle CORS for a non-preflight request. This means manipulating the `res` HTTP headers so that
/// the response is aligned with the `state`'s [CorsConfig].
///
/// If you are using the [Resource](crate::Resource) type (which is the recommended way), you'll never
/// have to call this method. However, if you are writing your own handler method, you might want to
/// call this after your request to add the required CORS headers.
///
/// For further information on CORS, read <https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS>.
pub fn handle_cors(state: &State, res: &mut Response<Body>) {
	let config = CorsConfig::try_borrow_from(state);
	if let Some(cfg) = config {
		let headers = res.headers_mut();

		// non-preflight requests require the Access-Control-Allow-Origin header
		if let Some(header) = cfg.origin.header_value(state) {
			headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, header);
		}

		// if the origin is copied over, we should tell the browser by specifying the Vary header
		if cfg.origin.varies() {
			let vary = headers
				.get(VARY)
				.map(|vary| format!("{},origin", vary.to_str().unwrap()));
			headers.insert(VARY, vary.as_deref().unwrap_or("origin").parse().unwrap());
		}

		// if we allow credentials, tell the browser
		if cfg.credentials {
			headers.insert(
				ACCESS_CONTROL_ALLOW_CREDENTIALS,
				HeaderValue::from_static("true")
			);
		}
	}
}

/// Add CORS routing for your path. This is required for handling preflight requests.
///
/// Example:
///
/// ```rust,no_run
/// # use gotham::{hyper::{Body, Method, Response}, router::builder::*};
/// # use gotham_restful::*;
/// build_simple_router(|router| {
/// 	// The handler that needs preflight handling
/// 	router.post("/foo").to(|state| {
/// 		let mut res: Response<Body> = unimplemented!();
/// 		handle_cors(&state, &mut res);
/// 		(state, res)
/// 	});
/// 	// Add preflight handling
/// 	router.cors("/foo", Method::POST);
/// });
/// ```
pub trait CorsRoute<C, P>
where
	C: PipelineHandleChain<P> + Copy + Send + Sync + 'static,
	P: RefUnwindSafe + Send + Sync + 'static
{
	/// Handle a preflight request on `path` for `method`. To configure the behaviour, use
	/// [CorsConfig].
	fn cors(&mut self, path: &str, method: Method);
}

pub(crate) fn cors_preflight_handler(state: State) -> (State, Response<Body>) {
	let config = CorsConfig::try_borrow_from(&state);

	// prepare the response
	let mut res = create_empty_response(&state, StatusCode::NO_CONTENT);
	let headers = res.headers_mut();
	let mut vary: Vec<HeaderName> = Vec::new();

	// copy the request method over to the response
	let method = HeaderMap::borrow_from(&state)
		.get(ACCESS_CONTROL_REQUEST_METHOD)
		.unwrap()
		.clone();
	headers.insert(ACCESS_CONTROL_ALLOW_METHODS, method);
	vary.push(ACCESS_CONTROL_REQUEST_METHOD);

	if let Some(cfg) = config {
		// if we allow any headers, copy them over
		if let Some(header) = cfg.headers.header_value(&state) {
			headers.insert(ACCESS_CONTROL_ALLOW_HEADERS, header);
		}

		// if the headers are copied over, we should tell the browser by specifying the Vary header
		if cfg.headers.varies() {
			vary.push(ACCESS_CONTROL_REQUEST_HEADERS);
		}

		// set the max age for the preflight cache
		if let Some(age) = config.map(|cfg| cfg.max_age) {
			headers.insert(ACCESS_CONTROL_MAX_AGE, age.into());
		}
	}

	// make sure the browser knows that this request was based on the method
	headers.insert(VARY, vary.join(",").parse().unwrap());

	handle_cors(&state, &mut res);
	(state, res)
}

impl<D, C, P> CorsRoute<C, P> for D
where
	D: DrawRoutes<C, P>,
	C: PipelineHandleChain<P> + Copy + Send + Sync + 'static,
	P: RefUnwindSafe + Send + Sync + 'static
{
	fn cors(&mut self, path: &str, method: Method) {
		let matcher = AccessControlRequestMethodMatcher::new(method);
		self.options(path)
			.extend_route_matcher(matcher)
			.to(cors_preflight_handler);
	}
}
