use gotham_restful::ResourceError;

#[derive(ResourceError)]
enum Error {
	#[display("I/O Error: {0}")]
	IoError(#[from] std::io::Error),

	#[status(INTERNAL_SERVER_ERROR)]
	#[display("Internal Server Error: {0}")]
	InternalServerError(String)
}

#[allow(deprecated)]
mod resource_error {
	use super::Error;
	use gotham::hyper::StatusCode;
	use gotham_restful::IntoResponseError;
	use mime::APPLICATION_JSON;

	#[test]
	fn io_error() {
		let err = Error::IoError(std::io::Error::last_os_error());
		let res = err.into_response_error().unwrap();
		assert_eq!(res.status, StatusCode::INTERNAL_SERVER_ERROR);
		assert_eq!(res.mime, Some(APPLICATION_JSON));
	}

	#[test]
	fn internal_server_error() {
		let err = Error::InternalServerError("Brocken".to_owned());
		assert_eq!(&format!("{}", err), "Internal Server Error: Brocken");

		let res = err.into_response_error().unwrap();
		assert_eq!(res.status, StatusCode::INTERNAL_SERVER_ERROR);
		assert_eq!(res.mime, None); // TODO shouldn't this be a json error message?
	}
}
