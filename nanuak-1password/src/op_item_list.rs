use tokio::process::Command;

use crate::types::Item;

pub async fn op_item_list() -> eyre::Result<Vec<Item>> {
    let mut cmd = Command::new("op");
    cmd.args(["item", "list", "--format=json"]);
    let output = cmd.output().await?;
    if !output.status.success() {
        eyre::bail!("op failed: {:?}", output);
    }
    let items: Vec<Item> = serde_json::from_slice(&output.stdout)?;
    Ok(items)
}
