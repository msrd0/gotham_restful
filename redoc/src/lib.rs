#![forbid(elided_lifetimes_in_paths, unsafe_code)]

//! Private implementation detail of the `gotham_restful` crate.

use base64::prelude::*;
use either::Either;
use sha2::{Digest, Sha256};
use std::{io::Write, iter};

#[doc(hidden)]
pub struct Redoc {
	/// HTML code.
	pub html: Vec<u8>,

	/// JS hash base64 encoded.
	pub script_hash: String
}

#[doc(hidden)]
pub fn html(spec: String) -> Redoc {
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
	let script_hash = BASE64_STANDARD.encode(script_hash.finalize());

	let mut html = Vec::<u8>::new();
	write!(
		html,
		concat!(
			"<!DOCTYPE HTML>",
			"<html>",
			"<head>",
			r#"<meta charset="utf-8"/>"#,
			r#"<meta name="viewport" content="width=device-width,initial-scale=1"/>"#,
			"</head>",
			r#"<body style="margin:0">"#,
			r#"<div id="spec" style="display:none">{}</div>"#,
			r#"<div id="redoc"></div>"#,
			r#"<script>{}</script>"#,
			"</body>",
			"</html>"
		),
		encoded_spec, script
	)
	.unwrap();

	Redoc { html, script_hash }
}
