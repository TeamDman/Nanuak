use crate::git::cloned_repos;
use diesel::prelude::*;
use chrono::NaiveDateTime;

#[derive(Debug, Insertable, Queryable, Selectable)]
#[diesel(table_name = cloned_repos)]
pub struct ClonedRepo {
    pub path: String,
    pub remotes: String,
    pub seen: NaiveDateTime,
}
