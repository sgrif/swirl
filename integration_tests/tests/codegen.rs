use crate::dummy_jobs::*;
use crate::test_guard::TestGuard;
use failure::Fallible;
use swirl::{JobsFailed, PerformError};

#[test]
fn generated_jobs_serialize_all_arguments_except_first() -> Fallible<()> {
    #[swirl::background_job]
    fn check_arg_equal_to_env(env: &String, arg: String) -> Result<(), PerformError> {
        if env == &arg {
            Ok(())
        } else {
            Err("arg wasn't env!".into())
        }
    }

    let runner = TestGuard::runner("a".to_string());
    let conn = runner.connection_pool().get()?;
    check_arg_equal_to_env("a".into()).enqueue(&conn)?;
    check_arg_equal_to_env("b".into()).enqueue(&conn)?;

    runner.run_all_pending_jobs()?;
    assert_eq!(Err(JobsFailed(1)), runner.check_for_failed_jobs());
    Ok(())
}

#[test]
fn jobs_with_args_but_no_env() -> Fallible<()> {
    #[swirl::background_job]
    fn assert_foo(arg: String) -> Result<(), PerformError> {
        if arg == "foo" {
            Ok(())
        } else {
            Err("arg wasn't foo!".into())
        }
    }

    let runner = TestGuard::dummy_runner();
    let conn = runner.connection_pool().get()?;
    assert_foo("foo".into()).enqueue(&conn)?;
    assert_foo("not foo".into()).enqueue(&conn)?;

    runner.run_all_pending_jobs()?;
    assert_eq!(Err(JobsFailed(1)), runner.check_for_failed_jobs());
    Ok(())
}
