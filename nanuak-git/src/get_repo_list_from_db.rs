use diesel::query_dsl::methods::OrderDsl;
use diesel::query_dsl::methods::SelectDsl;
use diesel::ExpressionMethods;
use diesel::PgConnection;
use diesel::RunQueryDsl;
use diesel::SelectableHelper;
use nanuak_schema::git_models::ClonedRepo;

pub async fn get_repo_list_from_db(conn: &mut PgConnection) -> eyre::Result<Vec<ClonedRepo>> {
    use nanuak_schema::git::cloned_repos::dsl as cr;
    let repos = cr::cloned_repos
        .select(ClonedRepo::as_select())
        .order(cr::seen.desc())
        .load::<ClonedRepo>(conn)?;
    Ok(repos)
}
