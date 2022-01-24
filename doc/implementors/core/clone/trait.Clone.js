(function() {var implementors = {};
implementors["gotham_restful"] = [{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"enum\" href=\"gotham_restful/enum.AuthStatus.html\" title=\"enum gotham_restful::AuthStatus\">AuthStatus</a>&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,&nbsp;</span>","synthetic":false,"types":["gotham_restful::auth::AuthStatus"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"enum\" href=\"gotham_restful/enum.AuthSource.html\" title=\"enum gotham_restful::AuthSource\">AuthSource</a>","synthetic":false,"types":["gotham_restful::auth::AuthSource"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"gotham_restful/struct.StaticAuthHandler.html\" title=\"struct gotham_restful::StaticAuthHandler\">StaticAuthHandler</a>","synthetic":false,"types":["gotham_restful::auth::StaticAuthHandler"]},{"text":"impl&lt;Data, Handler&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"gotham_restful/struct.AuthMiddleware.html\" title=\"struct gotham_restful::AuthMiddleware\">AuthMiddleware</a>&lt;Data, Handler&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Handler: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,&nbsp;</span>","synthetic":false,"types":["gotham_restful::auth::AuthMiddleware"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"enum\" href=\"gotham_restful/cors/enum.Origin.html\" title=\"enum gotham_restful::cors::Origin\">Origin</a>","synthetic":false,"types":["gotham_restful::cors::Origin"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"enum\" href=\"gotham_restful/cors/enum.Headers.html\" title=\"enum gotham_restful::cors::Headers\">Headers</a>","synthetic":false,"types":["gotham_restful::cors::Headers"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"gotham_restful/cors/struct.CorsConfig.html\" title=\"struct gotham_restful::cors::CorsConfig\">CorsConfig</a>","synthetic":false,"types":["gotham_restful::cors::CorsConfig"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"gotham_restful/struct.OpenapiInfo.html\" title=\"struct gotham_restful::OpenapiInfo\">OpenapiInfo</a>","synthetic":false,"types":["gotham_restful::openapi::builder::OpenapiInfo"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"gotham_restful/struct.NoopExtractor.html\" title=\"struct gotham_restful::NoopExtractor\">NoopExtractor</a>","synthetic":false,"types":["gotham_restful::endpoint::NoopExtractor"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"enum\" href=\"gotham_restful/enum.AuthError.html\" title=\"enum gotham_restful::AuthError\">AuthError</a>","synthetic":false,"types":["gotham_restful::response::auth_result::AuthError"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"gotham_restful/struct.NoContent.html\" title=\"struct gotham_restful::NoContent\">NoContent</a>","synthetic":false,"types":["gotham_restful::response::no_content::NoContent"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"gotham_restful/struct.Raw.html\" title=\"struct gotham_restful::Raw\">Raw</a>&lt;T&gt;","synthetic":false,"types":["gotham_restful::response::raw::Raw"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"gotham_restful/struct.Redirect.html\" title=\"struct gotham_restful::Redirect\">Redirect</a>","synthetic":false,"types":["gotham_restful::response::redirect::Redirect"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"gotham_restful/struct.Success.html\" title=\"struct gotham_restful::Success\">Success</a>&lt;T&gt;","synthetic":false,"types":["gotham_restful::response::success::Success"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()