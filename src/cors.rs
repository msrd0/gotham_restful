use gotham::{
    handler::HandlerFuture,
    hyper::{
		header::{ACCESS_CONTROL_ALLOW_ORIGIN, ORIGIN, HeaderMap, HeaderValue},
		Body, Method, Response
	},
    middleware::Middleware,
    state::{FromState, State},
};
use std::pin::Pin;

/**
Specify the allowed origins of the request. It is up to the browser to check the validity of the
origin. This, when sent to the browser, will indicate whether or not the request's origin was
allowed to make the request.
*/
#[derive(Clone, Debug)]
pub enum Origin
{
	/// Do not send any `Access-Control-Allow-Origin` headers.
    None,
	/// Send `Access-Control-Allow-Origin: *`. Note that browser will not send credentials.
    Star,
	/// Set the `Access-Control-Allow-Origin` header to a single origin.
    Single(String),
	/// Copy the `Origin` header into the `Access-Control-Allow-Origin` header.
	Copy
}

impl Default for Origin
{
	fn default() -> Self
	{
		Self::None
	}
}

impl Origin
{
	/// Get the header value for the `Access-Control-Allow-Origin` header.
	fn header_value(&self, state : &State) -> Option<HeaderValue>
	{
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
}

/**
This is the configuration that the CORS handler will follow. Its default configuration is basically
not to touch any responses, resulting in the browser's default behaviour.

To change settings, you need to put this type into gotham's [`State`]:

```rust,no_run
# use gotham::{router::builder::*, pipeline::{new_pipeline, single::single_pipeline}, state::State};
# use gotham_restful::*;
fn main() {
	let cors = CorsConfig {
    	origin: Origin::Star
	};
	let (chain, pipelines) = single_pipeline(new_pipeline().add(cors).build());
	gotham::start("127.0.0.1:8080", build_router(chain, pipelines, |route| {
		// your routing logic
	}));
}
```

This easy approach allows you to have one global cors configuration. If you prefer to have separate
configurations for different scopes, you need to register the middleware inside your routing logic:

```rust,no_run
# use gotham::{router::builder::*, pipeline::*, pipeline::set::*, state::State};
# use gotham_restful::*;
fn main() {
	let pipelines = new_pipeline_set();
	
	let cors_a = CorsConfig {
    	origin: Origin::Star
	};
	let (pipelines, chain_a) = pipelines.add(
		new_pipeline().add(cors_a).build()
	);
	
	let cors_b = CorsConfig {
    	origin: Origin::Copy
	};
	let (pipelines, chain_b) = pipelines.add(
		new_pipeline().add(cors_b).build()
	);
	
	let pipeline_set = finalize_pipeline_set(pipelines);
	gotham::start("127.0.0.1:8080", build_router((), pipeline_set, |route| {
		// routing without any cors config
		route.with_pipeline_chain((chain_a, ()), |route| {
			// routing with cors config a
		});
		route.with_pipeline_chain((chain_b, ()), |route| {
			// routing with cors config b
		});
	}));
}
```

 [`State`]: ../gotham/state/struct.State.html
*/
#[derive(Clone, Debug, Default, NewMiddleware, StateData)]
pub struct CorsConfig
{
	pub origin : Origin
}

impl Middleware for CorsConfig
{
    fn call<Chain>(self, mut state : State, chain : Chain) -> Pin<Box<HandlerFuture>>
    where
        Chain : FnOnce(State) -> Pin<Box<HandlerFuture>>
    {
        state.put(self);
        chain(state)
    }
}

/**
Handle CORS for a non-preflight request. This means manipulating the `res` HTTP headers so that
the response is aligned with the `state`'s [`CorsConfig`].

If you are using the [`Resource`] type (which is the recommended way), you'll never have to call
this method. However, if you are writing your own handler method, you might want to call this
after your request to add the required CORS headers.

For further information on CORS, read https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS.

 [`CorsConfig`]: ./struct.CorsConfig.html
*/
pub fn handle_cors(state : &State, res : &mut Response<Body>)
{
	let method = Method::borrow_from(state);
	let config = CorsConfig::try_borrow_from(state);
	
    // non-preflight requests require nothing other than the Access-Control-Allow-Origin header
	if let Some(header) = config.and_then(|cfg| cfg.origin.header_value(state))
	{
		res.headers_mut().insert(ACCESS_CONTROL_ALLOW_ORIGIN, header);
	}
}
