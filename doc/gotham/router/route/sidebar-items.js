initSidebarItems({"enum":[["Delegation","Indicates whether this `Route` will dispatch the request to an inner `Router` instance. To support inner `Router` instances which handle a subtree, the `Dispatcher` stores additional context information."]],"mod":[["dispatch","Defines the route `Dispatcher` and supporting types."],["matcher","Defines the type `RouteMatcher` and default implementations."]],"struct":[["ExtractorFailed","Returned in the `Err` variant from `extract_query_string` or `extract_request_path`, this signals that the extractor has failed and the request should not proceed."],["Extractors","Extractors used by `RouteImpl` to acquire request data and change into a type safe form for use by `Middleware` and `Handler` implementations."],["RouteImpl","Concrete type for a route in a Gotham web application. Values of this type are created by the `gotham::router::builder` API and held internally in the `Router` for dispatching requests."]],"trait":[["Route","Values of the `Route` type are used by the `Router` to conditionally dispatch a request after matching the path segments successfully. The steps taken in dispatching to a `Route` are:"]]});