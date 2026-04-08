use std::path::PathBuf;

use md_cli_test::Tester;
use todo_app::config::{DEFAULT_CONFIG_FILE_NAME, ROOT_CONFIG_ENV_KEY};

pub fn run_test_cases(md_file_path: impl Into<PathBuf>) -> anyhow::Result<()> {
    Tester::new(md_file_path)
        .with_env(ROOT_CONFIG_ENV_KEY, format!("./{DEFAULT_CONFIG_FILE_NAME}"))
        .run()?;

    Ok(())
}
