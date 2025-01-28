use diesel::upsert::excluded;
use chrono::Utc;
use diesel::prelude::*;
use nanuak_schema::git_models::NewClonedRepo;

pub fn upsert_cloned_repos(
    conn: &mut PgConnection,
    discovered: &[(std::path::PathBuf, String)],
) -> eyre::Result<()> {
    use nanuak_schema::git::cloned_repos::dsl as cr;

    for (local_path, origin_url) in discovered {
        let now = Utc::now().naive_utc();

        let new_row = NewClonedRepo {
            path: local_path.to_string_lossy().to_string(),
            remotes: origin_url.clone(),
            seen: now,
        };
        
        diesel::insert_into(cr::cloned_repos)
            .values(&new_row)
            .on_conflict(cr::path) // path is PK
            .do_update()
            .set((
                cr::remotes.eq(excluded(cr::remotes)),
                cr::seen.eq(excluded(cr::seen)),
            ))
            .execute(conn)?;
    }
    Ok(())
}
