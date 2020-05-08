use trybuild::TestCases;

#[test]
#[ignore]
fn trybuild_ui()
{
	let t = TestCases::new();
	
	// always enabled
	t.compile_fail("tests/ui/from_body_enum.rs");
	t.compile_fail("tests/ui/method_async_state.rs");
	t.compile_fail("tests/ui/method_for_unknown_resource.rs");
	t.compile_fail("tests/ui/method_no_resource.rs");
	t.compile_fail("tests/ui/method_self.rs");
	t.compile_fail("tests/ui/method_too_few_args.rs");
	t.compile_fail("tests/ui/method_too_many_args.rs");
	t.compile_fail("tests/ui/method_unsafe.rs");
}
