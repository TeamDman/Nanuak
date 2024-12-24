use crate::files_schema::files::*;
use diesel::prelude::*;

#[derive(Insertable)]
#[diesel(table_name = files)]
pub struct NewFile<'a> {
    pub path: &'a str,
    pub file_size: i64,
    pub hash_value: &'a str,
    pub hash_algorithm: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = requests)]
pub struct NewRequest<'a> {
    pub file_id: i32,
    pub request_type: &'a str,
    pub model: &'a str,
}
