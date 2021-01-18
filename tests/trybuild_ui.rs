use trybuild::TestCases;

#[test]
#[ignore]
fn trybuild_ui() {
	let t = TestCases::new();

	// always enabled
	t.compile_fail("tests/ui/endpoint/*.rs");
	t.compile_fail("tests/ui/from_body/*.rs");
	t.compile_fail("tests/ui/resource/*.rs");

	// require the openapi feature
	if cfg!(feature = "openapi") {
		t.compile_fail("tests/ui/openapi_type/*.rs");
	}
}
