initSidebarItems({"attr":[["create",""],["delete",""],["delete_all",""],["endpoint",""],["read",""],["read_all",""],["search",""],["update",""],["update_all",""]],"derive":[["FromBody",""],["RequestBody",""],["Resource",""],["ResourceError",""]],"enum":[["AuthError","This is an error type that always yields a 403 Forbidden response. This type is best used in combination with [AuthSuccess] or [AuthResult]."],["AuthErrorOrOther","This is an error type that either yields a 403 Forbidden respone if produced from an authentication error, or delegates to another error type. This type is best used with [AuthResult]."],["AuthSource","The source of the authentication token in the request."],["AuthStatus","The authentication status returned by the auth middleware for each request."]],"mod":[["cors",""]],"struct":[["AuthMiddleware","This is the auth middleware. To use it, first make sure you have the `auth` feature enabled. Then simply add it to your pipeline and request it inside your handler:"],["NoContent","This is the return type of a resource that doesn't actually return something. It will result in a 204 No Content answer by default. You don't need to use this type directly if using the function attributes:"],["NoopExtractor","A no-op extractor that can be used as a default type for [Endpoint::Placeholders] and [Endpoint::Params]."],["OpenapiInfo",""],["Raw","This type can be used both as a raw request body, as well as as a raw response. However, all types of request bodies are accepted by this type. It is therefore recommended to derive your own type from [RequestBody] and only use this when you need to return a raw response. This is a usage example that simply returns its body:"],["Redirect","This is the return type of a resource that only returns a redirect. It will result in a 303 See Other answer, meaning the redirect will always result in a GET request on the target."],["Response","A response, used to create the final gotham response from."],["StaticAuthHandler","An [AuthHandler] returning always the same secret. See [AuthMiddleware] for a usage example."],["Success","This can be returned from a resource when there is no cause of an error."]],"trait":[["AuthHandler","This trait will help the auth middleware to determine the validity of an authentication token."],["DrawResourceRoutes","This trait allows to draw routes within an resource. Use this only inside the [Resource::setup] method."],["DrawResourceRoutesWithSchema","This trait allows to draw routes within an resource. Use this only inside the [Resource::setup] method."],["DrawResources","This trait adds the `resource` method to gotham's routing. It allows you to register any RESTful [Resource] with a path."],["DrawResourcesWithSchema","This trait adds the `resource` method to gotham's routing. It allows you to register any RESTful [Resource] with a path."],["Endpoint",""],["EndpointWithSchema",""],["FromBody","This trait should be implemented for every type that can be built from an HTTP request body plus its media type."],["GetOpenapi","This trait adds the `openapi_spec` and `openapi_doc` method to an OpenAPI-aware router."],["IntoResponse","This trait needs to be implemented by every type returned from an endpoint to to provide the response."],["IntoResponseError",""],["IntoResponseWithSchema","A trait provided to convert a resource's result to json, and provide an OpenAPI schema to the router. This trait is implemented for all types that implement [IntoResponse] and [ResponseSchema]."],["RequestBody","A type that can be used inside a request body. Implemented for every type that is deserializable with serde. If the `openapi` feature is used, it must also be of type OpenapiType."],["Resource","This trait must be implemented for every resource. It allows you to register the different endpoints that can be handled by this resource to be registered with the underlying router."],["ResourceType",""],["ResourceWithSchema","This trait must be implemented for every resource. It allows you to register the different endpoints that can be handled by this resource to be registered with the underlying router."],["ResponseBody","A type that can be used inside a response body. Implemented for every type that is serializable with serde. If the `openapi` feature is used, it must also be of type OpenapiType."],["ResponseSchema","Additional details for [IntoResponse] to be used with an OpenAPI-aware router."],["WithOpenapi","This trait adds the `with_openapi` method to gotham's routing. It turns the default router into one that will only allow RESTful resources, but record them and generate an OpenAPI specification on request."]],"type":[["AuthResult","This return type can be used to wrap any type implementing IntoResponse that can only be returned if the client is authenticated. Otherwise, an empty 403 Forbidden response will be issued."],["AuthSuccess","This return type can be used to wrap any type implementing IntoResponse that can only be returned if the client is authenticated. Otherwise, an empty 403 Forbidden response will be issued."],["AuthValidation",""]]});