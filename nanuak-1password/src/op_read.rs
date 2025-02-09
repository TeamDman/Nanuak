use eyre::Context;
use tokio::process::Command;

pub async fn op_read(reference: &str) -> eyre::Result<String> {
    let mut cmd = Command::new("op");
    cmd.args(["read", "--no-newline"]);
    cmd.arg(reference);
    let output = cmd.output().await?;
    if !output.status.success() {
        eyre::bail!("op failed: {:?}", output);
    }
    let secret = String::from_utf8(output.stdout)
        .wrap_err("Failed to interpret 1password output as utf-8 string")?;
    Ok(secret)
}
