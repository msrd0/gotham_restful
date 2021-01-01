use gotham::hyper::{Body, StatusCode};
use mime::{Mime, APPLICATION_JSON};

/// A response, used to create the final gotham response from.
#[derive(Debug)]
pub struct Response {
	pub status: StatusCode,
	pub body: Body,
	pub mime: Option<Mime>
}

impl Response {
	/// Create a new [Response] from raw data.
	#[must_use = "Creating a response is pointless if you don't use it"]
	pub fn new<B: Into<Body>>(status: StatusCode, body: B, mime: Option<Mime>) -> Self {
		Self {
			status,
			body: body.into(),
			mime
		}
	}

	/// Create a [Response] with mime type json from already serialized data.
	#[must_use = "Creating a response is pointless if you don't use it"]
	pub fn json<B: Into<Body>>(status: StatusCode, body: B) -> Self {
		Self {
			status,
			body: body.into(),
			mime: Some(APPLICATION_JSON)
		}
	}

	/// Create a _204 No Content_ [Response].
	#[must_use = "Creating a response is pointless if you don't use it"]
	pub fn no_content() -> Self {
		Self {
			status: StatusCode::NO_CONTENT,
			body: Body::empty(),
			mime: None
		}
	}

	/// Create an empty _403 Forbidden_ [Response].
	#[must_use = "Creating a response is pointless if you don't use it"]
	pub fn forbidden() -> Self {
		Self {
			status: StatusCode::FORBIDDEN,
			body: Body::empty(),
			mime: None
		}
	}

	#[cfg(test)]
	pub(crate) fn full_body(mut self) -> Result<Vec<u8>, <Body as gotham::hyper::body::HttpBody>::Error> {
		use futures_executor::block_on;
		use gotham::hyper::body::to_bytes;

		let bytes: &[u8] = &block_on(to_bytes(&mut self.body))?;
		Ok(bytes.to_vec())
	}
}
