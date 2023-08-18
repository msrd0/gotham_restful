var searchIndex = JSON.parse('{\
"gotham_restful":{"doc":"This crate is an extension to the popular gotham web …","t":"DEIDGEEGGNNQQNCCIIIIIIQQQNIYINIIINDDDNQQQQQQDDIYIYYIDIIDDNNILLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLAXLLLXXLLLKKXLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLKLLLLCKKCLLLLLLLLLKKLLLLLLLLLLLLLKLLLLLKLLLKLLMLLLLLLLLLLLLLLLKKLLKMXXKKKKLLLLLLXKKLKLLLLLLLLLLLLMMLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLXXKKMMLLLLLLKNNDIENNNENNLLLLLLLLLLLLLLLKMLLLLLLLLLFMLLLMLMLLLLLLLLLLLLLLLL","n":["AuthError","AuthErrorOrOther","AuthHandler","AuthMiddleware","AuthResult","AuthSource","AuthStatus","AuthSuccess","AuthValidation","Authenticated","AuthorizationHeader","Body","Body","Cookie","CorsConfig","CorsRoute","DrawResourceRoutes","DrawResourceRoutesWithSchema","DrawResources","DrawResourcesWithSchema","Endpoint","EndpointWithSchema","Err","Err","Err","Forbidden","FromBody","FromBody","GetOpenapi","Header","IntoResponse","IntoResponseError","IntoResponseWithSchema","Invalid","NoContent","NoopExtractor","OpenapiInfo","Other","Output","Output","Params","Params","Placeholders","Placeholders","Raw","Redirect","RequestBody","RequestBody","Resource","Resource","ResourceError","ResourceWithSchema","Response","ResponseBody","ResponseSchema","StaticAuthHandler","Success","Unauthenticated","Unknown","WithOpenapi","accepted_types","accepted_types","accepted_types","accepted_types","as_mut","as_ref","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow_from","borrow_from","borrow_from","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut_from","borrow_mut_from","borrow_mut_from","call","clone","clone","clone","clone","clone","clone","clone","clone","clone","clone","clone","clone","clone_into","clone_into","clone_into","clone_into","clone_into","clone_into","clone_into","clone_into","clone_into","clone_into","clone_into","clone_into","cors","create","default","default","default","delete","delete_all","description","description","deserialize","endpoint","endpoint","endpoint","extend","fmt","fmt","fmt","fmt","fmt","fmt","fmt","fmt","fmt","fmt","fmt","fmt","fmt","forbidden","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from_array","from_body","from_body","from_body","from_source","from_vec","gotham","handle","handle","handle_cors","has_placeholders","has_placeholders","has_placeholders","has_placeholders","header","header","header","headers_mut","headers_mut","http_method","http_method","into","into","into","into","into","into","into","into","into","into","into","into","into","into_response","into_response","into_response","into_response","into_response","into_response","into_response_error","into_response_error","into_response_error","json","jwt_secret","jwt_secret","mime","mime","needs_body","needs_body","needs_body","needs_body","needs_params","needs_params","needs_params","needs_params","new","new","new","new","new_middleware","no_content","ok","openapi_doc","openapi_spec","operation_id","operation_id","operation_verb","raw","read","read_all","resource","resource","schema","schema","schema","schema","schema","schema","schema","schema","search","setup","setup","status","status_codes","status_codes","status_codes","status_codes","status_codes","status_codes","status_codes","supported_types","supported_types","supported_types","take_from","take_from","take_from","title","to","to_owned","to_owned","to_owned","to_owned","to_owned","to_owned","to_owned","to_owned","to_owned","to_owned","to_owned","to_owned","try_borrow_from","try_borrow_from","try_borrow_from","try_borrow_mut_from","try_borrow_mut_from","try_borrow_mut_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_take_from","try_take_from","try_take_from","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","update","update_all","uri","uri","urls","version","visit_type","visit_type","wants_auth","wants_auth","wants_auth","wants_auth","with_openapi","Copy","Copy","CorsConfig","CorsRoute","Headers","List","None","None","Origin","Single","Star","borrow","borrow","borrow","borrow_from","borrow_mut","borrow_mut","borrow_mut","borrow_mut_from","call","clone","clone","clone","clone_into","clone_into","clone_into","cors","credentials","default","default","default","fmt","fmt","fmt","from","from","from","handle_cors","headers","into","into","into","max_age","new_middleware","origin","take_from","to_owned","to_owned","to_owned","try_borrow_from","try_borrow_mut_from","try_from","try_from","try_from","try_into","try_into","try_into","try_take_from","type_id","type_id","type_id"],"q":[[0,"gotham_restful"],[343,"gotham_restful::cors"]],"d":["This is an error type that always yields a <em>403 Forbidden</em> …","This is an error type that either yields a <em>403 Forbidden</em> …","This trait will help the auth middleware to determine the …","This is the auth middleware. To use it, first make sure …","This return type can be used to wrap any type implementing …","The source of the authentication token in the request.","The authentication status returned by the auth middleware …","This return type can be used to wrap any type implementing …","","The request has been performed with a valid …","Take the token from the HTTP Authorization header. This is …","The type to parse the body into. Use <code>()</code> if <code>needs_body()</code> …","The type to parse the body into. Use <code>()</code> if <code>needs_body()</code> …","Take the token from a cookie with the given name.","","","This trait allows to draw routes within an resource. Use …","This trait allows to draw routes within an resource. Use …","This trait adds the <code>resource</code> method to gotham’s routing. …","This trait adds the <code>resource</code> method to gotham’s routing. …","","","","","The error type returned by the conversion if it was …","","This trait should be implemented for every type that can …","","This trait adds the <code>openapi_spec</code> and <code>openapi_doc</code> method to …","Take the token from a header with the given name.","This trait needs to be implemented by every type returned …","","A trait provided to convert a resource’s result to json, …","The request has been performed with an invalid …","This is the return type of a resource that doesn’t …","A no-op extractor that can be used as a default type for …","","","The output type that provides the response.","The output type that provides the response.","The type that parses the request parameters. Use …","The type that parses the request parameters. Use …","The type that parses the URI placeholders. Use …","The type that parses the URI placeholders. Use …","This type can be used both as a raw request body, as well …","This is the return type of a resource that only returns a …","A type that can be used inside a request body. Implemented …","","This trait must be implemented for every resource. It …","","","This trait must be implemented for every resource. It …","A response, used to create the final gotham response from.","A type that can be used inside a response body. …","Additional details for IntoResponse to be used with an …","An AuthHandler returning always the same secret. See …","This can be returned from a resource when there is no …","The request has been performed without any kind of …","The auth status is unknown. This is likely because no …","This trait adds the <code>with_openapi</code> method to gotham’s …","Return a list of supported mime types.","Return a list of supported mime types.","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","Add a description to the openapi specification. Usually …","Add a description to the openapi specification. Usually …","","","","","","","","","","","","","","","","","","","Create an empty <em>403 Forbidden</em> Response.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","","Returns the argument unchanged.","","","Returns the argument unchanged.","","Returns the argument unchanged.","Returns the argument unchanged.","","Returns the argument unchanged.","","Returns the argument unchanged.","","Perform the conversion.","","","","","","The handler for this endpoint.","The handler for this endpoint.","","Returns <code>true</code> <em>iff</em> the URI contains placeholders. <code>false</code> by …","Returns <code>true</code> <em>iff</em> the URI contains placeholders. <code>false</code> by …","Returns <code>true</code> <em>iff</em> the URI contains placeholders. <code>false</code> by …","Returns <code>true</code> <em>iff</em> the URI contains placeholders. <code>false</code> by …","Set a custom HTTP header. If a header with this name was …","Set a custom HTTP header. If a header with this name was …","Add an HTTP header to the Response.","Allow manipulating HTTP headers.","Allow manipulating HTTP headers.","The HTTP Verb of this endpoint.","The HTTP Verb of this endpoint.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Turn this into a response that can be returned to the …","This will always be a <em>204 No Content</em> together with an …","","","","","","","","Create a Response with mime type json from already …","Return the SHA256-HMAC secret used to verify the JWT token.","","Return the mime type of this Response.","","Returns <code>true</code> <em>iff</em> the request body should be parsed. <code>false</code> …","Returns <code>true</code> <em>iff</em> the request body should be parsed. <code>false</code> …","Returns <code>true</code> <em>iff</em> the request body should be parsed. <code>false</code> …","Returns <code>true</code> <em>iff</em> the request body should be parsed. <code>false</code> …","Returns <code>true</code> <em>iff</em> the request parameters should be parsed. …","Returns <code>true</code> <em>iff</em> the request parameters should be parsed. …","Returns <code>true</code> <em>iff</em> the request parameters should be parsed. …","Returns <code>true</code> <em>iff</em> the request parameters should be parsed. …","","","","Create a new Response from raw data.","","Create a <em>204 No Content</em> Response.","","Register a GET route to <code>path</code> that returns the OpenAPI …","Register a GET route to <code>path</code> that returns the OpenAPI …","Replace the automatically generated operation id with a …","Replace the automatically generated operation id with a …","The verb used for generating an operation id if …","","","","","","","Return the schema of the response for the given status …","","","Returns the schema of the <code>()</code> type.","","","","","Register all methods handled by this resource with the …","Register all methods handled by this resource with the …","Return the status code of this Response.","","All status codes returned by this response. Returns …","All status codes returned by this response. Returns …","","","","","Return all types that are supported as content types. Use …","Return all types that are supported as content types. Use …","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","The URI that this endpoint listens on in gotham’s format.","The URI that this endpoint listens on in gotham’s format.","","","","","Returns <code>true</code> if the request wants to know the auth status …","Returns <code>true</code> if the request wants to know the auth status …","Returns <code>true</code> if the request wants to know the auth status …","Returns <code>true</code> if the request wants to know the auth status …","","Copy the <code>Origin</code> header into the <code>Access-Control-Allow-Origin</code>…","Copy the <code>Access-Control-Request-Headers</code> header into the …","This is the configuration that the CORS handler will …","Add CORS routing for your path. This is required for …","Specify the allowed headers of the request. It is up to …","Set the <code>Access-Control-Allow-Headers</code> header to the …","Do not send any <code>Access-Control-Allow-Origin</code> headers.","Do not send any <code>Access-Control-Allow-Headers</code> headers.","Specify the allowed origins of the request. It is up to …","Set the <code>Access-Control-Allow-Origin</code> header to a single …","Send <code>Access-Control-Allow-Origin: *</code>. Note that browser …","","","","","","","","","","","","","","","","Handle a preflight request on <code>path</code> for <code>method</code>. To …","Whether or not the request may be made with supplying …","","","","","","","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Handle CORS for a non-preflight request. This means …","The allowed headers.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","The amount of seconds that the preflight request can be …","","The allowed origins.","","","","","","","","","","","","","","","",""],"i":[0,0,0,0,0,0,0,0,0,17,18,66,67,18,0,0,0,0,0,0,0,0,68,69,70,23,0,0,0,18,0,0,0,17,0,0,0,23,66,67,66,67,66,67,0,0,0,0,0,0,0,0,0,0,0,0,0,17,17,0,69,69,24,26,5,5,17,18,19,11,20,21,22,23,24,5,25,26,36,17,18,21,17,18,19,11,20,21,22,23,24,5,25,26,36,17,18,21,11,17,18,19,11,20,21,22,23,24,5,25,26,17,18,19,11,20,21,22,23,24,5,25,26,0,0,24,25,26,0,0,67,67,21,57,58,0,21,17,18,19,11,20,21,22,23,24,5,25,26,36,36,17,18,19,11,20,21,22,23,23,23,23,24,24,5,25,26,26,26,36,19,70,21,5,11,19,0,66,67,0,66,66,67,67,24,26,36,24,26,66,67,17,18,19,11,20,21,22,23,24,5,25,26,36,69,24,5,25,26,36,68,22,23,36,10,19,36,5,66,66,67,67,66,66,67,67,11,22,5,36,11,36,17,71,71,67,67,67,5,0,0,72,73,68,74,22,23,24,5,25,26,0,75,76,36,68,74,74,22,23,24,25,77,77,21,17,18,21,20,25,17,18,19,11,20,21,22,23,24,5,25,26,17,18,21,17,18,21,17,18,19,11,20,21,22,23,24,5,25,26,36,17,18,19,11,20,21,22,23,24,5,25,26,36,17,18,21,17,18,19,11,20,21,22,23,24,5,25,26,36,0,0,66,67,20,20,21,5,66,66,67,67,78,64,65,0,0,0,65,64,65,0,64,64,64,65,63,63,64,65,63,63,63,64,65,63,64,65,63,79,63,64,65,63,64,65,63,64,65,63,0,63,64,65,63,63,63,63,63,64,65,63,63,63,64,65,63,64,65,63,63,64,65,63],"f":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,[[],[[3,[[2,[1]]]]]],[[],[[3,[[2,[1]]]]]],[[],[[3,[[2,[1]]]]]],[[],[[3,[[2,[1]]]]]],[[[5,[4]]]],[[[5,[6]]]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[7],[7],[7],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[7],[7],[7],[[[11,[[0,[8,9]],[10,[[0,[8,9]]]]]],7,12],[[15,[[14,[13]]]]]],[[[17,[[0,[16,9]]]]],[[17,[[0,[16,9]]]]]],[18,18],[19,19],[[[11,[16]]],[[11,[16]]]],[20,20],[21,21],[22,22],[[[23,[16]]],[[23,[16]]]],[24,24],[[[5,[16]]],[[5,[16]]]],[25,25],[[[26,[16]]],[[26,[16]]]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],0,0,[[],24],[[],25],[[],[[26,[27]]]],0,0,[[],[[3,[28]]]],[[],[[3,[28]]]],[29,[[30,[21]]]],[[]],[[]],0,[[7,[32,[31]]]],[[[17,[[0,[33,9]]]],34],35],[[18,34],35],[[19,34],35],[[[11,[33,33]],34],35],[[20,34],35],[[21,34],35],[[22,34],35],[[[23,[33]],34],35],[[24,34],35],[[[5,[33]],34],35],[[25,34],35],[[[26,[33]],34],35],[[36,34],35],[[],36],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[22,23],[[]],[37],[[[0,[0,38]]],23],[[]],[[],24],[[]],[[]],[[],26],[[]],[37],[[]],[[[40,[39]]],19],[[41,1],30],[[41,1],30],[[41,1],[[30,[[5,[[42,[[40,[39]]]]]]]]]],[18,[[11,[[0,[8,9]],[0,[[10,[[0,[8,9]]]],27]]]]]],[[[2,[39]]],19],0,[[7,3],43],[[7,3],43],0,[[],44],[[],44],[[],44],[[],44],[[24,45,46]],[[26,45,46]],[[36,47,46]],[24,48],[26,48],[[],49],[[],49],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[],[[43,[[30,[36]]]]]],[24,[[15,[[14,[50]]]]]],[[[5,[[38,[31]]]]],[[15,[[14,[50]]]]]],[25,[[43,[[30,[36]]]]]],[[[26,[51]]],[[15,[[14,[50]]]]]],[36,[[43,[[30,[36]]]]]],[[],[[30,[36]]]],[22,[[30,[36]]]],[23,[[30,[36]]]],[[52,[38,[31]]],36],[[7,12],[[3,[[2,[39]]]]]],[[19,7,12],[[3,[[2,[39]]]]]],[36,[[3,[1]]]],0,[[],44],[[],44],[[],44],[[],44],[[],44],[[],44],[[],44],[[],44],[[18,53,[10,[[0,[8,9]]]]],[[11,[[0,[8,9]],[10,[[0,[8,9]]]]]]]],[[[38,[28]]],22],[1,5],[[52,[38,[31]],[3,[1]]],36],[11,[[54,[11]]]],[[],36],[[[17,[9]]],[[30,[9,22]]]],[55],[55],[[],[[3,[28]]]],[[],[[3,[28]]]],[[],[[3,[55]]]],0,0,0,[55],[55],[52,56],[52,56],[52,56],[52,56],[52,56],[52,56],[52,56],[52,56],0,[57],[58],[36,52],[[],[[2,[52]]]],[[],[[2,[52]]]],[[],[[2,[52]]]],[[],[[2,[52]]]],[[],[[2,[52]]]],[[],[[2,[52]]]],[[],[[2,[52]]]],[[],[[3,[[2,[1]]]]]],[[],[[3,[[2,[1]]]]]],[[],[[3,[[2,[1,59]]]]]],[7],[7],[7],0,0,[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[7,3],[7,3],[7,3],[7,3],[7,3],[7,3],[[],30],[[],30],[[],30],[[],30],[[],30],[[],30],[[],30],[[],30],[[],30],[[],30],[[],30],[[],30],[[],30],[[],30],[[],30],[[],30],[[],30],[[],30],[[],30],[[],30],[[],30],[[],30],[[],30],[[],30],[[],30],[[],30],[7,3],[7,3],[7,3],[[],60],[[],60],[[],60],[[],60],[[],60],[[],60],[[],60],[[],60],[[],60],[[],60],[[],60],[[],60],[[],60],0,0,[[],[[61,[55]]]],[[],[[61,[55]]]],0,0,[62],[62],[[],44],[[],44],[[],44],[[],44],[[20,12]],0,0,0,0,0,0,0,0,0,0,0,[[]],[[]],[[]],[7],[[]],[[]],[[]],[7],[[63,7,12],[[15,[[14,[13]]]]]],[64,64],[65,65],[63,63],[[]],[[]],[[]],[[55,49]],0,[[],64],[[],65],[[],63],[[64,34],35],[[65,34],35],[[63,34],35],[[]],[[]],[[]],[[7,[32,[31]]]],0,[[]],[[]],[[]],0,[63,[[54,[63]]]],0,[7],[[]],[[]],[[]],[7,3],[7,3],[[],30],[[],30],[[],30],[[],30],[[],30],[[],30],[7,3],[[],60],[[],60],[[],60]],"c":[],"p":[[3,"Mime"],[3,"Vec"],[4,"Option"],[8,"AsMut"],[3,"Raw"],[8,"AsRef"],[3,"State"],[8,"DeserializeOwned"],[8,"Send"],[8,"AuthHandler"],[3,"AuthMiddleware"],[8,"FnOnce"],[6,"HandlerFuture"],[3,"Box"],[3,"Pin"],[8,"Clone"],[4,"AuthStatus"],[4,"AuthSource"],[3,"StaticAuthHandler"],[3,"OpenapiInfo"],[3,"NoopExtractor"],[3,"AuthError"],[4,"AuthErrorOrOther"],[3,"NoContent"],[3,"Redirect"],[3,"Success"],[8,"Default"],[3,"String"],[8,"Deserializer"],[4,"Result"],[3,"Body"],[3,"Response"],[8,"Debug"],[3,"Formatter"],[6,"Result"],[3,"Response"],[15,"never"],[8,"Into"],[15,"u8"],[15,"slice"],[3,"Bytes"],[8,"From"],[6,"BoxFuture"],[15,"bool"],[8,"IntoHeaderName"],[3,"HeaderValue"],[3,"HeaderName"],[3,"HeaderMap"],[3,"Method"],[8,"Future"],[8,"ResponseBody"],[3,"StatusCode"],[6,"AuthValidation"],[6,"Result"],[15,"str"],[3,"OpenapiSchema"],[8,"DrawResourceRoutes"],[8,"DrawResourceRoutesWithSchema"],[3,"Global"],[3,"TypeId"],[4,"Cow"],[8,"Visitor"],[3,"CorsConfig"],[4,"Origin"],[4,"Headers"],[8,"Endpoint"],[8,"EndpointWithSchema"],[8,"IntoResponseError"],[8,"IntoResponse"],[8,"FromBody"],[8,"GetOpenapi"],[8,"DrawResources"],[8,"DrawResourcesWithSchema"],[8,"ResponseSchema"],[8,"Resource"],[8,"ResourceWithSchema"],[8,"RequestBody"],[8,"WithOpenapi"],[8,"CorsRoute"]]}\
}');
if (typeof window !== 'undefined' && window.initSearch) {window.initSearch(searchIndex)};
if (typeof exports !== 'undefined') {exports.searchIndex = searchIndex};
