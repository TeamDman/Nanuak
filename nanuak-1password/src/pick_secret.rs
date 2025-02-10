use cloud_terrastodon_core_user_input::prelude::Choice;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use cloud_terrastodon_core_user_input::prelude::pick;
use itertools::Itertools;
use tracing::info;

use crate::op_item_get::op_item_get;
use crate::op_item_list::op_item_list;
use crate::types::Field;

pub async fn pick_secret() -> eyre::Result<Field> {
    let items = op_item_list().await?;
    let chosen = pick(FzfArgs {
        choices: items
            .into_iter()
            .map(|item| Choice {
                key: format!("{} - {}", item.vault.name, item.title),
                value: item,
            })
            .collect_vec(),
        header: None,
        prompt: None,
    })?;
    info!(
        "You chose: {} - {} (id is {})",
        chosen.vault.name, chosen.title, chosen.id
    );
    let item = op_item_get(&chosen.id).await?;
    let field = pick(FzfArgs {
        choices: item
            .fields
            .unwrap_or_default()
            .into_iter()
            .map(|field| Choice {
                key: field.label.clone(),
                value: field,
            })
            .collect_vec(),
        header: None,
        prompt: None,
    })?;
    info!("You chose: {:?}", field.reference);
    Ok(field.value)
}
