use std::collections::HashSet;

use cloud_terrastodon_core_user_input::prelude::Choice;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use itertools::Itertools;
use nanuak_picking::picker::pick_many;
use strum::VariantArray;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[derive(Debug, VariantArray, Hash, Eq, PartialEq, Clone, Copy)]
enum Word {
    Dog,
    Cat,
    Spider,
    Wolf,
    Human,
    Elephant,
    Snake,
    Whale,
    House,
    Car,
    Airplane,
    Tractor,
    Boat,
    Refrigerator,
    Toaster,
}

#[tokio::test]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::DEBUG.into())
                .from_env_lossy(),
        )
        .without_time()
        .init();

    let choices = Word::VARIANTS
        .iter()
        .map(|animal| Choice {
            key: format!("{:?}", animal),
            value: animal,
        })
        .collect_vec();

    let chosen = pick_many(FzfArgs {
        choices,
        header: Some("animal".to_string()),
        ..Default::default()
    })
    .await?
    .into_iter()
    .map(|x| x.value)
    .collect::<HashSet<_>>();

    let expected = [
        Word::Dog,
        Word::Cat,
        Word::Spider,
        Word::Wolf,
        Word::Human,
        Word::Elephant,
        Word::Snake,
        Word::Whale,
    ]
    .iter()
    .collect::<HashSet<_>>();

    assert_eq!(chosen, expected);
    Ok(())
}
