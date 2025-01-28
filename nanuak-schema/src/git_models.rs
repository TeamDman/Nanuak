use crate::git::cloned_repos;
use diesel::prelude::*;
use chrono::NaiveDateTime;

#[derive(Debug, Insertable)]
#[diesel(table_name = cloned_repos)]
pub struct NewClonedRepo {
    pub path: String,
    pub remotes: String,
    pub seen: NaiveDateTime,
}
