#![forbid(elided_lifetimes_in_paths, unsafe_code)]

//! Private implementation detail of the `gotham_restful` crate.

use either::Either;
use gotham::{
	handler::HandlerError,
	helpers::http::response::{create_empty_response, create_response},
	hyper::{
		header::{
			HeaderMap, HeaderValue, CACHE_CONTROL, CONTENT_SECURITY_POLICY, ETAG, IF_NONE_MATCH,
			REFERRER_POLICY, X_CONTENT_TYPE_OPTIONS
		},
		Body, Response, StatusCode
	},
	mime::TEXT_HTML_UTF_8,
	state::State
};
use sha2::{Digest, Sha256};
use std::{io::Write, iter};

#[doc(hidden)]
pub fn handler(state: &State, spec: String) -> Result<Response<Body>, HandlerError> {
	let encoded_spec = spec
		.chars()
		.flat_map(|c| match c {
			'&' => Either::Left("&amp;".chars()),
			'<' => Either::Left("&lt;".chars()),
			'>' => Either::Left("&gt;".chars()),
			c => Either::Right(iter::once(c))
		})
		.collect::<String>();

	let script = include_str!("script.min.js");
	let mut script_hash = Sha256::new();
	script_hash.update(&script);
	let script_hash = base64::encode(script_hash.finalize());

	let mut buf = Vec::<u8>::new();
	write!(
		buf,
		r#"<!DOCTYPE HTML><html><head><meta charset="utf-8"/><meta name="viewport" content="width=device-width,initial-scale=1"/></head>"#
	)?;
	write!(
		buf,
		r#"<body style="margin:0"><div id="spec" style="display:none">{}</div><div id="redoc"></div><script>{}</script></body></html>"#,
		encoded_spec, script
	)?;

	let mut etag = Sha256::new();
	etag.update(&buf);
	let etag = format!("\"{}\"", base64::encode(etag.finalize()));

	if state
		.borrow::<HeaderMap>()
		.get(IF_NONE_MATCH)
		.map_or(false, |header| header.as_bytes() == etag.as_bytes())
	{
		let res = create_empty_response(state, StatusCode::NOT_MODIFIED);
		return Ok(res);
	}

	let mut res = create_response(state, StatusCode::OK, TEXT_HTML_UTF_8, buf);
	let headers = res.headers_mut();
	headers.insert(
		CACHE_CONTROL,
		HeaderValue::from_static("public,max-age=2592000")
	);
	headers.insert(
		CONTENT_SECURITY_POLICY,
		format!(
			"default-src 'none';base-uri 'none';script-src 'unsafe-inline' https://cdn.jsdelivr.net 'sha256-{script_hash}' 'strict-dynamic';style-src 'unsafe-inline' https://fonts.googleapis.com;font-src https://fonts.gstatic.com;connect-src 'self';img-src blob: data:",
		).parse().unwrap()
	);
	headers.insert(ETAG, etag.parse().unwrap());
	headers.insert(REFERRER_POLICY, HeaderValue::from_static("no-referrer"));
	headers.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
	Ok(res)
}
