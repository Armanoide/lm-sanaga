use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create session table
        manager
            .create_table(
                Table::create()
                    .table(Session::Table)
                    .if_not_exists()
                    .col(pk_auto(Session::Id))
                    .col(string(Session::Name))
                    .col(
                        ColumnDef::new(Session::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create conversation table
        manager
            .create_table(
                Table::create()
                    .table(Conversation::Table)
                    .if_not_exists()
                    .col(pk_auto(Conversation::Id))
                    .col(string_null(Conversation::Name))
                    .col(ColumnDef::new(Conversation::SessionId).integer().not_null())
                    .col(
                        ColumnDef::new(Conversation::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_conversation_session")
                            .from(Conversation::Table, Conversation::SessionId)
                            .to(Session::Table, Session::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create message table
        manager
            .create_table(
                Table::create()
                    .table(Message::Table)
                    .if_not_exists()
                    .col(pk_auto(Message::Id))
                    .col(string_null(Message::Role))
                    .col(text_null(Message::Content))
                    .col(double_null(Message::GenerationDuration))
                    .col(double_null(Message::PromptTps))
                    .col(double_null(Message::GenerationTps))
                    .col(ColumnDef::new(Message::ConversationId).integer().not_null())
                    .col(
                        ColumnDef::new(Message::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_message_conversation")
                            .from(Message::Table, Message::ConversationId)
                            .to(Conversation::Table, Conversation::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Session {
    Table,
    Id,
    Name,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Conversation {
    Table,
    Id,
    Name,
    SessionId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Message {
    Table,
    Id,
    Role,
    Content,
    GenerationDuration,
    PromptTps,
    GenerationTps,
    ConversationId,
    CreatedAt,
}
