use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct RunArgs {
    /// Hook name to run
    pub hook: String,
    
    /// Additional arguments for the hook
    pub args: Vec<String>,
}

pub async fn execute(args: RunArgs, verbosity_level: u8) -> Result<()> {
    use crate::config::GuardyConfig;
    use crate::hooks::HookExecutor;
    
    // Load configuration
    let config = GuardyConfig::load(None, None::<&()>, verbosity_level)?;
    
    // Create hook executor and run the hook
    let executor = HookExecutor::new(config)?;
    executor.execute(&args.hook, args.args).await
}