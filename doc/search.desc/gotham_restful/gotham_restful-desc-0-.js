searchState.loadedDescShard("gotham_restful", 0, "This crate is an extension to the popular gotham web …\nThis is an error type that always yields a <em>403 Forbidden</em> …\nThis is an error type that either yields a <em>403 Forbidden</em> …\nThis trait will help the auth middleware to determine the …\nThis is the auth middleware. To use it, first make sure …\nThis return type can be used to wrap any type implementing …\nThe source of the authentication token in the request.\nThe authentication status returned by the auth middleware …\nThis return type can be used to wrap any type implementing …\nThe request has been performed with a valid …\nTake the token from the HTTP Authorization header. This is …\nThe type to parse the body into. Use <code>()</code> if <code>needs_body()</code> …\nThe type to parse the body into. Use <code>()</code> if <code>needs_body()</code> …\nTake the token from a cookie with the given name.\nThis trait allows to draw routes within an resource. Use …\nThis trait allows to draw routes within an resource. Use …\nThis trait adds the <code>resource</code> method to gotham’s routing. …\nThis trait adds the <code>resource</code> method to gotham’s routing. …\nContains the error value\nContains the error value\nThe error type returned by the conversion if it was …\nThis trait should be implemented for every type that can …\nAutomatically generate the operation id based on path and …\nThis trait adds the <code>openapi_spec</code> and <code>openapi_doc</code> method to …\nTake the token from a header with the given name.\nThis trait needs to be implemented by every type returned …\nA trait provided to convert a resource’s result to json, …\nThe request has been performed with an invalid …\nUse the provided operation id.\nThis is the return type of a resource that doesn’t …\nA no-op extractor that can be used as a default type for …\nContains the success value\nContains the success value\nThe output type that provides the response.\nThe output type that provides the response.\nThe type that parses the request parameters. Use …\nThe type that parses the request parameters. Use …\nThe type that parses the URI placeholders. Use …\nThe type that parses the URI placeholders. Use …\nThis type can be used both as a raw request body, as well …\nThis is the return type of a resource that only returns a …\nA type that can be used inside a request body. Implemented …\nThis trait must be implemented for every resource. It …\nThis trait must be implemented for every resource. It …\nA response, used to create the final gotham response from.\nA type that can be used inside a response body. …\nAdditional details for IntoResponse to be used with an …\nAutomatically generate the operation id based on path and …\nAn AuthHandler returning always the same secret. See …\nThis can be returned from a resource when there is no …\nThe request has been performed without any kind of …\nThe auth status is unknown. This is likely because no …\nThis trait adds the <code>with_openapi</code> method to gotham’s …\nReturn a list of supported mime types.\nReturn a list of supported mime types.\nThe validation will check that the <code>alg</code> of the header is …\nValidation will check that the <code>aud</code> field is a member of the\nAdd a description to the openapi specification. Usually …\nAdd a description to the openapi specification. Usually …\nCreate an empty <em>403 Forbidden</em> Response.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nPerform the conversion.\nThe handler for this endpoint.\nThe handler for this endpoint.\nReturns <code>true</code> <em>iff</em> the URI contains placeholders. <code>false</code> by …\nReturns <code>true</code> <em>iff</em> the URI contains placeholders. <code>false</code> by …\nReturns <code>true</code> <em>iff</em> the URI contains placeholders. <code>false</code> by …\nReturns <code>true</code> <em>iff</em> the URI contains placeholders. <code>false</code> by …\nSet a custom HTTP header. If a header with this name was …\nSet a custom HTTP header. If a header with this name was …\nAdd an HTTP header to the Response.\nAllow manipulating HTTP headers.\nAllow manipulating HTTP headers.\nThe HTTP Verb of this endpoint.\nThe HTTP Verb of this endpoint.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nTurn this into a response that can be returned to the …\nThis will always be a <em>204 No Content</em> together with an …\nIf it contains a value, the validation will check that the …\nCreate a Response with mime type json from already …\nReturn the SHA256-HMAC secret used to verify the JWT token.\nAdd some leeway (in seconds) to the <code>exp</code> and <code>nbf</code> validation …\nReturn the mime type of this Response.\nReturns <code>true</code> <em>iff</em> the request body should be parsed. <code>false</code> …\nReturns <code>true</code> <em>iff</em> the request body should be parsed. <code>false</code> …\nReturns <code>true</code> <em>iff</em> the request body should be parsed. <code>false</code> …\nReturns <code>true</code> <em>iff</em> the request body should be parsed. <code>false</code> …\nReturns <code>true</code> <em>iff</em> the request parameters should be parsed. …\nReturns <code>true</code> <em>iff</em> the request parameters should be parsed. …\nReturns <code>true</code> <em>iff</em> the request parameters should be parsed. …\nReturns <code>true</code> <em>iff</em> the request parameters should be parsed. …\nCreate a new Response from raw data.\nCreate a <em>204 No Content</em> Response.\nRegister a GET route to <code>path</code> that returns the OpenAPI …\nRegister a GET route to <code>path</code> that returns the OpenAPI …\nReplace the automatically generated operation id with a …\nReplace the automatically generated operation id with a …\nThe verb used for generating an operation id if …\nReject a token some time (in seconds) before the <code>exp</code> to …\nWhich claims are required to be present before starting …\nReturn the schema of the response for the given status …\nReturns the schema of the <code>()</code> type.\nRegister all methods handled by this resource with the …\nRegister all methods handled by this resource with the …\nReturn the status code of this Response.\nAll status codes returned by this response. Returns …\nAll status codes returned by this response. Returns …\nIf it contains a value, the validation will check that the …\nReturn all types that are supported as content types. Use …\nReturn all types that are supported as content types. Use …\nThe URI that this endpoint listens on in gotham’s format.\nThe URI that this endpoint listens on in gotham’s format.\nWhether to validate the <code>aud</code> field.\nWhether to validate the <code>exp</code> field.\nWhether to validate the <code>nbf</code> field.\nReturns <code>true</code> if the request wants to know the auth status …\nReturns <code>true</code> if the request wants to know the auth status …\nReturns <code>true</code> if the request wants to know the auth status …\nReturns <code>true</code> if the request wants to know the auth status …\nCopy the <code>Origin</code> header into the <code>Access-Control-Allow-Origin</code>…\nCopy the <code>Access-Control-Request-Headers</code> header into the …\nThis is the configuration that the CORS handler will …\nAdd CORS routing for your path. This is required for …\nSpecify the allowed headers of the request. It is up to …\nSet the <code>Access-Control-Allow-Headers</code> header to the …\nDo not send any <code>Access-Control-Allow-Origin</code> headers.\nDo not send any <code>Access-Control-Allow-Headers</code> headers.\nSpecify the allowed origins of the request. It is up to …\nSet the <code>Access-Control-Allow-Origin</code> header to a single …\nSend <code>Access-Control-Allow-Origin: *</code>. Note that browser …\nHandle a preflight request on <code>path</code> for <code>method</code>. To …\nWhether or not the request may be made with supplying …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nHandle CORS for a non-preflight request. This means …\nThe allowed headers.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nThe amount of seconds that the preflight request can be …\nThe allowed origins.")