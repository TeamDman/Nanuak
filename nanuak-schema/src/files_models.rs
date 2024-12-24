pub mod files_models {
    use crate::files_schema::files::*;
    use diesel::prelude::*;

    #[derive(Insertable)]
    #[diesel(table_name = files)]
    struct NewFile<'a> {
        path: &'a str,
        file_size: i64,
        hash_value: &'a str,
        hash_algorithm: &'a str,
    }

    #[derive(Insertable)]
    #[diesel(table_name = requests)]
    struct NewRequest<'a> {
        file_id: i32,
        request_type: &'a str,
        model: &'a str,
    }
}
