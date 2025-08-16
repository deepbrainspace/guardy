use anyhow::Result;
use clap::Args;

#[derive(Args, Clone)]
pub struct RunArgs {
    /// Hook name to run
    pub hook: String,

    /// Additional arguments for the hook
    pub args: Vec<String>,
}

pub async fn execute(args: RunArgs) -> Result<()> {
    use crate::config::CONFIG;
    use crate::hooks::HookExecutor;

    // Create hook executor and run the hook
    let executor = HookExecutor::new();
    executor.execute(&args.hook, &args.args).await
}
