use trybuild::TestCases;

#[test]
#[ignore]
fn trybuild_ui() {
	let t = TestCases::new();
	t.compile_fail("tests/ui/endpoint/*.rs");
	t.compile_fail("tests/ui/from_body/*.rs");
	t.compile_fail("tests/ui/resource/*.rs");
}
