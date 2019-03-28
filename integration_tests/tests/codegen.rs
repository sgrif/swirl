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

#[test]
fn env_can_have_any_name() {
    #[swirl::background_job]
    fn env_with_different_name(environment: &String) -> Result<(), swirl::PerformError> {
        assert_eq!(environment, "my environment");
        Ok(())
    }

    let runner = TestGuard::runner(String::from("my environment"));
    let conn = runner.connection_pool().get().unwrap();
    env_with_different_name().enqueue(&conn).unwrap();

    runner.run_all_pending_jobs().unwrap();
    runner.assert_no_failed_jobs().unwrap();
}

#[test]
#[forbid(unused_imports)]
fn test_imports_only_used_in_job_body_are_not_warned_as_unused() {
    use std::io::prelude::*;

    #[swirl::background_job]
    fn uses_trait_import() -> Result<(), swirl::PerformError> {
        let mut buf = Vec::new();
        buf.write_all(b"foo")?;
        let s = String::from_utf8(buf)?;
        assert_eq!(s, "foo");
        Ok(())
    }

    let runner = TestGuard::dummy_runner();
    let conn = runner.connection_pool().get().unwrap();
    uses_trait_import().enqueue(&conn).unwrap();

    runner.run_all_pending_jobs().unwrap();
    runner.assert_no_failed_jobs().unwrap();
}
