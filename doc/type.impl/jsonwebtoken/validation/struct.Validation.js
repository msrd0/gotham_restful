(function() {var type_impls = {
"gotham_restful":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Validation\" class=\"impl\"><a href=\"#impl-Validation\" class=\"anchor\">§</a><h3 class=\"code-header\">impl Validation</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.new\" class=\"method\"><h4 class=\"code-header\">pub fn <a class=\"fn\">new</a>(alg: Algorithm) -&gt; Validation</h4></section></summary><div class=\"docblock\"><p>Create a default validation setup allowing the given alg</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.set_audience\" class=\"method\"><h4 class=\"code-header\">pub fn <a class=\"fn\">set_audience</a>&lt;T&gt;(&amp;mut self, items: &amp;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.slice.html\">[T]</a>)<div class=\"where\">where\n    T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/alloc/string/trait.ToString.html\" title=\"trait alloc::string::ToString\">ToString</a>,</div></h4></section></summary><div class=\"docblock\"><p><code>aud</code> is a collection of one or more acceptable audience members\nThe simple usage is <code>set_audience(&amp;[&quot;some aud name&quot;])</code></p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.set_issuer\" class=\"method\"><h4 class=\"code-header\">pub fn <a class=\"fn\">set_issuer</a>&lt;T&gt;(&amp;mut self, items: &amp;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.slice.html\">[T]</a>)<div class=\"where\">where\n    T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/alloc/string/trait.ToString.html\" title=\"trait alloc::string::ToString\">ToString</a>,</div></h4></section></summary><div class=\"docblock\"><p><code>iss</code> is a collection of one or more acceptable issuers members\nThe simple usage is <code>set_issuer(&amp;[&quot;some iss name&quot;])</code></p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.set_required_spec_claims\" class=\"method\"><h4 class=\"code-header\">pub fn <a class=\"fn\">set_required_spec_claims</a>&lt;T&gt;(&amp;mut self, items: &amp;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.slice.html\">[T]</a>)<div class=\"where\">where\n    T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/alloc/string/trait.ToString.html\" title=\"trait alloc::string::ToString\">ToString</a>,</div></h4></section></summary><div class=\"docblock\"><p>Which claims are required to be present for this JWT to be considered valid.\nThe only values that will be considered are “exp”, “nbf”, “aud”, “iss”, “sub”.\nThe simple usage is <code>set_required_spec_claims(&amp;[&quot;exp&quot;, &quot;nbf&quot;])</code>.\nIf you want to have an empty set, do not use this function - set an empty set on the struct\nparam directly.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.insecure_disable_signature_validation\" class=\"method\"><h4 class=\"code-header\">pub fn <a class=\"fn\">insecure_disable_signature_validation</a>(&amp;mut self)</h4></section></summary><div class=\"docblock\"><p>Whether to validate the JWT cryptographic signature.\nDisabling validation is dangerous, only do it if you know what you’re doing.\nWith validation disabled you should not trust any of the values of the claims.</p>\n</div></details></div></details>",0,"gotham_restful::auth::AuthValidation"],["<section id=\"impl-StructuralPartialEq-for-Validation\" class=\"impl\"><a href=\"#impl-StructuralPartialEq-for-Validation\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/marker/trait.StructuralPartialEq.html\" title=\"trait core::marker::StructuralPartialEq\">StructuralPartialEq</a> for Validation</h3></section>","StructuralPartialEq","gotham_restful::auth::AuthValidation"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-Validation\" class=\"impl\"><a href=\"#impl-Debug-for-Validation\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for Validation</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.77.2/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/1.77.2/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.77.2/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.unit.html\">()</a>, <a class=\"struct\" href=\"https://doc.rust-lang.org/1.77.2/core/fmt/struct.Error.html\" title=\"struct core::fmt::Error\">Error</a>&gt;</h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/1.77.2/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","gotham_restful::auth::AuthValidation"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Default-for-Validation\" class=\"impl\"><a href=\"#impl-Default-for-Validation\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a> for Validation</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.default\" class=\"method trait-impl\"><a href=\"#method.default\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.77.2/core/default/trait.Default.html#tymethod.default\" class=\"fn\">default</a>() -&gt; Validation</h4></section></summary><div class='docblock'>Returns the “default value” for a type. <a href=\"https://doc.rust-lang.org/1.77.2/core/default/trait.Default.html#tymethod.default\">Read more</a></div></details></div></details>","Default","gotham_restful::auth::AuthValidation"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-PartialEq-for-Validation\" class=\"impl\"><a href=\"#impl-PartialEq-for-Validation\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> for Validation</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.eq\" class=\"method trait-impl\"><a href=\"#method.eq\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.77.2/core/cmp/trait.PartialEq.html#tymethod.eq\" class=\"fn\">eq</a>(&amp;self, other: &amp;Validation) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.bool.html\">bool</a></h4></section></summary><div class='docblock'>This method tests for <code>self</code> and <code>other</code> values to be equal, and is used\nby <code>==</code>.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.ne\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/1.77.2/src/core/cmp.rs.html#242\">source</a></span><a href=\"#method.ne\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.77.2/core/cmp/trait.PartialEq.html#method.ne\" class=\"fn\">ne</a>(&amp;self, other: <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.reference.html\">&amp;Rhs</a>) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.bool.html\">bool</a></h4></section></summary><div class='docblock'>This method tests for <code>!=</code>. The default implementation is almost always\nsufficient, and should not be overridden without very good reason.</div></details></div></details>","PartialEq","gotham_restful::auth::AuthValidation"],["<section id=\"impl-Eq-for-Validation\" class=\"impl\"><a href=\"#impl-Eq-for-Validation\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a> for Validation</h3></section>","Eq","gotham_restful::auth::AuthValidation"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Clone-for-Validation\" class=\"impl\"><a href=\"#impl-Clone-for-Validation\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for Validation</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone\" class=\"method trait-impl\"><a href=\"#method.clone\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.77.2/core/clone/trait.Clone.html#tymethod.clone\" class=\"fn\">clone</a>(&amp;self) -&gt; Validation</h4></section></summary><div class='docblock'>Returns a copy of the value. <a href=\"https://doc.rust-lang.org/1.77.2/core/clone/trait.Clone.html#tymethod.clone\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone_from\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/1.77.2/src/core/clone.rs.html#169\">source</a></span><a href=\"#method.clone_from\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.77.2/core/clone/trait.Clone.html#method.clone_from\" class=\"fn\">clone_from</a>(&amp;mut self, source: <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.reference.html\">&amp;Self</a>)</h4></section></summary><div class='docblock'>Performs copy-assignment from <code>source</code>. <a href=\"https://doc.rust-lang.org/1.77.2/core/clone/trait.Clone.html#method.clone_from\">Read more</a></div></details></div></details>","Clone","gotham_restful::auth::AuthValidation"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()