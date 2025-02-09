use tokio::process::Command;

use crate::types::Item;

pub async fn op_item_get(item_id: &str) -> eyre::Result<Item> {
    let mut cmd = Command::new("op");
    cmd.args(["item", "get", "--format=json"]);
    cmd.arg(item_id);
    let output = cmd.output().await?;
    if !output.status.success() {
        eyre::bail!("op failed: {:?}", output);
    }
    let item: Item = serde_json::from_slice(&output.stdout)?;
    Ok(item)
}
