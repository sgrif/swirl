use crate::dummy_jobs::*;
use crate::test_guard::TestGuard;
use swirl::PerformError;

#[test]
#[should_panic(expected = "1 jobs failed")]
fn generated_jobs_serialize_all_arguments_except_first() {
    #[swirl::background_job]
    fn check_arg_equal_to_env(env: &String, arg: String) -> Result<(), PerformError> {
        if env == &arg {
            Ok(())
        } else {
            Err("arg wasn't env!".into())
        }
    }

    let runner = TestGuard::runner("a".to_string());
    let conn = runner.connection_pool().get().unwrap();
    check_arg_equal_to_env("a".into()).enqueue(&conn).unwrap();
    check_arg_equal_to_env("b".into()).enqueue(&conn).unwrap();

    runner.run_all_pending_jobs().unwrap();
    runner.assert_no_failed_jobs().unwrap();
}

#[test]
#[should_panic(expected = "1 jobs failed")]
fn jobs_with_args_but_no_env() {
    #[swirl::background_job]
    fn assert_foo(arg: String) -> Result<(), PerformError> {
        if arg == "foo" {
            Ok(())
        } else {
            Err("arg wasn't foo!".into())
        }
    }

    let runner = TestGuard::dummy_runner();
    let conn = runner.connection_pool().get().unwrap();
    assert_foo("foo".into()).enqueue(&conn).unwrap();
    assert_foo("not foo".into()).enqueue(&conn).unwrap();

    runner.run_all_pending_jobs().unwrap();
    runner.assert_no_failed_jobs().unwrap();
}
