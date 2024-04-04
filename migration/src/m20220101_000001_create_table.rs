use sea_orm_migration::prelude::*;
use std::{env, fs};

#[derive(DeriveMigrationName)]
pub struct Migration;

/// postgresql里 migration 会在事务里执行
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db: &SchemaManagerConnection<'_> = manager.get_connection();
        let mut current_dir = env::current_dir().unwrap();
        current_dir.push("migration/src/create_table_up.sql");
        let sql_content =
            fs::read_to_string(current_dir).expect("failed to read create_table_up file");
        db.execute_unprepared(sql_content.as_str()).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db: &SchemaManagerConnection<'_> = manager.get_connection();
                let mut current_dir = env::current_dir().unwrap();
        current_dir.push("migration/src/create_table_.sql");
        let sql_content = fs::read_to_string(current_dir)
            .expect("failed to read create_table_down file");
        db.execute_unprepared(sql_content.as_str()).await?;
        Ok(())
    }
}
