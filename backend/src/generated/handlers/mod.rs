use crate::generated::*;
use crate::generated::executor::*;
use std::collections::HashMap;

impl Account {
    #[tracing::instrument]
    pub async fn create<'c, E>(executor: E, item: &AccountNew, user_id: uuid::Uuid) -> Result<Account, AccountCreateError>
    where
        E: PgExecutor<'c>,
    {
        let res = sqlx::query_as::<_, Account>("INSERT INTO account (amount, description, name, type, user_id) VALUES ($1, $2, $3, $4, $5) RETURNING *")
            .bind(&item.amount)
            .bind(&item.description)
            .bind(&item.name)
            .bind(&item.r#type)
            .bind(&user_id)
            .fetch_one(executor)
            .await;
        match res {
            Ok(v) => Ok(v),
            Err(e) => {
                if let sqlx::Error::Database(db_err) = &e {
                    if let Some(c) = db_err.constraint() {
                        if c == "account_name_user_unique" { return Err(AccountCreateError::AccountNameUserUnique); }
                        if c == "user" { return Err(AccountCreateError::UserFk); }
                    }
                }
                Err(AccountCreateError::Database(e))
            }
        }
    }

    #[tracing::instrument]
    pub async fn find<'c, E>(executor: E, id: uuid::Uuid, user_id: uuid::Uuid) -> Result<Account, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        sqlx::query_as::<_, Account>("SELECT * FROM account WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL")
            .bind(id)
            .bind(user_id)
            .fetch_one(executor)
            .await
    }

    #[tracing::instrument]
    pub async fn update<'c, E>(executor: E, id: uuid::Uuid, user_id: uuid::Uuid, item: &AccountUpdate) -> Result<Account, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new("UPDATE account SET " );
        let mut has_updates = false;
        let mut separated = qb.separated(", ");
        if let Some(value) = &item.amount { separated.push("amount = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.deleted_at { separated.push("deleted_at = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.description { separated.push("description = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.name { separated.push("name = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.r#type { separated.push("type = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.user_id { separated.push("user_id = ").push_bind(value); has_updates = true; }
        if !has_updates {
            // Can't call Self::find with a generic executor easily, so we query directly
            return sqlx::query_as("SELECT * FROM account WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL")
                .bind(id).bind(user_id).fetch_one(executor).await;
        }
        qb.push(" WHERE id = " ).push_bind(id).push(" AND user_id = ").push_bind(user_id).push(" RETURNING *");
        let query = qb.build_query_as::<Account>();
        query.fetch_one(executor).await
    }

    #[tracing::instrument]
    pub async fn delete<'c, E>(executor: E, id: uuid::Uuid, user_id: uuid::Uuid) -> Result<u64, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let res = sqlx::query("UPDATE account SET deleted_at = now() WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(executor)
            .await?;
        Ok(res.rows_affected())
    }

    #[tracing::instrument]
    pub async fn list<'c, E>(executor: E, user_id: uuid::Uuid, pagination: Option<Pagination>) -> Result<Vec<Account>, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new("SELECT * FROM account");
        qb.push(" WHERE deleted_at IS NULL AND user_id = ").push_bind(user_id);
        if let Some(p) = pagination {
            qb.push(" LIMIT " ).push_bind(p.limit);
            qb.push(" OFFSET " ).push_bind(p.offset);
        }
        qb.build_query_as::<Account>().fetch_all(executor).await
    }

    // --- Relational Helper: has many ---
    #[tracing::instrument]
    pub async fn find_transactions<'c, E>(&self, executor: E, pagination: Option<Pagination>) -> Result<Vec<Transaction>, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new("SELECT * FROM transaction");
        qb.push(" WHERE ");
        qb.push("deleted_at IS NULL");
        qb.push(" AND ");
        qb.push("account_id = ").push_bind(self.id);
        if let Some(p) = pagination {
            qb.push(" LIMIT " ).push_bind(p.limit);
            qb.push(" OFFSET " ).push_bind(p.offset);
        }
        qb.build_query_as::<Transaction>().fetch_all(executor).await
    }

    // --- Relational Helper: has many ---
    #[tracing::instrument]
    pub async fn find_recurringtransactions<'c, E>(&self, executor: E, pagination: Option<Pagination>) -> Result<Vec<RecurringTransaction>, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new("SELECT * FROM recurringtransaction");
        qb.push(" WHERE ");
        qb.push("deleted_at IS NULL");
        qb.push(" AND ");
        qb.push("account_id = ").push_bind(self.id);
        if let Some(p) = pagination {
            qb.push(" LIMIT " ).push_bind(p.limit);
            qb.push(" OFFSET " ).push_bind(p.offset);
        }
        qb.build_query_as::<RecurringTransaction>().fetch_all(executor).await
    }

    // --- Eager Loading Helper ---
    #[tracing::instrument]
    pub async fn find_by_ids<'c, E>(executor: E, ids: &[uuid::Uuid]) -> Result<Vec<Account>, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        sqlx::query_as("SELECT * FROM account WHERE id = ANY($1) AND deleted_at IS NULL")
            .bind(ids)
            .fetch_all(executor)
            .await
    }

}

impl Budget {
    #[tracing::instrument]
    pub async fn create<'c, E>(executor: E, item: &BudgetNew, user_id: uuid::Uuid) -> Result<Budget, BudgetCreateError>
    where
        E: PgExecutor<'c>,
    {
        let res = sqlx::query_as::<_, Budget>("INSERT INTO budget (amount, category_id, description, end_date, name, period, start_date, user_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *")
            .bind(&item.amount)
            .bind(&item.category_id)
            .bind(&item.description)
            .bind(&item.end_date)
            .bind(&item.name)
            .bind(&item.period)
            .bind(&item.start_date)
            .bind(&user_id)
            .fetch_one(executor)
            .await;
        match res {
            Ok(v) => Ok(v),
            Err(e) => {
                if let sqlx::Error::Database(db_err) = &e {
                    if let Some(c) = db_err.constraint() {
                        if c == "budget_name_user_unique" { return Err(BudgetCreateError::BudgetNameUserUnique); }
                        if c == "category" { return Err(BudgetCreateError::CategoryFk); }
                        if c == "user" { return Err(BudgetCreateError::UserFk); }
                    }
                }
                Err(BudgetCreateError::Database(e))
            }
        }
    }

    #[tracing::instrument]
    pub async fn find<'c, E>(executor: E, id: uuid::Uuid, user_id: uuid::Uuid) -> Result<Budget, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        sqlx::query_as::<_, Budget>("SELECT * FROM budget WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL")
            .bind(id)
            .bind(user_id)
            .fetch_one(executor)
            .await
    }

    #[tracing::instrument]
    pub async fn update<'c, E>(executor: E, id: uuid::Uuid, user_id: uuid::Uuid, item: &BudgetUpdate) -> Result<Budget, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new("UPDATE budget SET " );
        let mut has_updates = false;
        let mut separated = qb.separated(", ");
        if let Some(value) = &item.amount { separated.push("amount = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.category_id { separated.push("category_id = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.deleted_at { separated.push("deleted_at = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.description { separated.push("description = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.end_date { separated.push("end_date = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.name { separated.push("name = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.period { separated.push("period = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.start_date { separated.push("start_date = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.user_id { separated.push("user_id = ").push_bind(value); has_updates = true; }
        if !has_updates {
            // Can't call Self::find with a generic executor easily, so we query directly
            return sqlx::query_as("SELECT * FROM budget WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL")
                .bind(id).bind(user_id).fetch_one(executor).await;
        }
        qb.push(" WHERE id = " ).push_bind(id).push(" AND user_id = ").push_bind(user_id).push(" RETURNING *");
        let query = qb.build_query_as::<Budget>();
        query.fetch_one(executor).await
    }

    #[tracing::instrument]
    pub async fn delete<'c, E>(executor: E, id: uuid::Uuid, user_id: uuid::Uuid) -> Result<u64, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let res = sqlx::query("UPDATE budget SET deleted_at = now() WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(executor)
            .await?;
        Ok(res.rows_affected())
    }

    #[tracing::instrument]
    pub async fn list<'c, E>(executor: E, user_id: uuid::Uuid, pagination: Option<Pagination>) -> Result<Vec<Budget>, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new("SELECT * FROM budget");
        qb.push(" WHERE deleted_at IS NULL AND user_id = ").push_bind(user_id);
        if let Some(p) = pagination {
            qb.push(" LIMIT " ).push_bind(p.limit);
            qb.push(" OFFSET " ).push_bind(p.offset);
        }
        qb.build_query_as::<Budget>().fetch_all(executor).await
    }

    // --- Eager Loading Helper ---
    #[tracing::instrument]
    pub async fn find_by_ids<'c, E>(executor: E, ids: &[uuid::Uuid]) -> Result<Vec<Budget>, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        sqlx::query_as("SELECT * FROM budget WHERE id = ANY($1) AND deleted_at IS NULL")
            .bind(ids)
            .fetch_all(executor)
            .await
    }

}

impl Category {
    #[tracing::instrument]
    pub async fn create<'c, E>(executor: E, item: &CategoryNew) -> Result<Category, CategoryCreateError>
    where
        E: PgExecutor<'c>,
    {
        let res = sqlx::query_as::<_, Category>("INSERT INTO category (description, icon, name, type, user_id) VALUES ($1, $2, $3, $4, $5) RETURNING *")
            .bind(&item.description)
            .bind(&item.icon)
            .bind(&item.name)
            .bind(&item.r#type)
            .bind(&item.user_id)
            .fetch_one(executor)
            .await;
        match res {
            Ok(v) => Ok(v),
            Err(e) => {
                if let sqlx::Error::Database(db_err) = &e {
                    if let Some(c) = db_err.constraint() {
                        if c == "category_name_user_unique" { return Err(CategoryCreateError::CategoryNameUserUnique); }
                        if c == "user" { return Err(CategoryCreateError::UserFk); }
                    }
                }
                Err(CategoryCreateError::Database(e))
            }
        }
    }

    #[tracing::instrument]
    pub async fn find<'c, E>(executor: E, id: uuid::Uuid) -> Result<Category, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        sqlx::query_as::<_, Category>("SELECT * FROM category WHERE id = $1 AND deleted_at IS NULL")
            .bind(id)
            .fetch_one(executor)
            .await
    }

    #[tracing::instrument]
    pub async fn update<'c, E>(executor: E, id: uuid::Uuid, item: &CategoryUpdate) -> Result<Category, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new("UPDATE category SET " );
        let mut has_updates = false;
        let mut separated = qb.separated(", ");
        if let Some(value) = &item.deleted_at { separated.push("deleted_at = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.description { separated.push("description = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.icon { separated.push("icon = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.name { separated.push("name = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.r#type { separated.push("type = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.user_id { separated.push("user_id = ").push_bind(value); has_updates = true; }
        if !has_updates {
            return sqlx::query_as("SELECT * FROM category WHERE id = $1 AND deleted_at IS NULL")
                .bind(id).fetch_one(executor).await;
        }
        qb.push(" WHERE id = " ).push_bind(id).push(" RETURNING *");
        let query = qb.build_query_as::<Category>();
        query.fetch_one(executor).await
    }

    #[tracing::instrument]
    pub async fn delete<'c, E>(executor: E, id: uuid::Uuid) -> Result<u64, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let res = sqlx::query("UPDATE category SET deleted_at = now() WHERE id = $1")
            .bind(id)
            .execute(executor)
            .await?;
        Ok(res.rows_affected())
    }

    #[tracing::instrument]
    pub async fn list<'c, E>(executor: E, pagination: Option<Pagination>) -> Result<Vec<Category>, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new("SELECT * FROM category");
        qb.push(" WHERE deleted_at IS NULL");
        if let Some(p) = pagination {
            qb.push(" LIMIT " ).push_bind(p.limit);
            qb.push(" OFFSET " ).push_bind(p.offset);
        }
        qb.build_query_as::<Category>().fetch_all(executor).await
    }

    // --- Relational Helper: has many ---
    #[tracing::instrument]
    pub async fn find_transactions<'c, E>(&self, executor: E, pagination: Option<Pagination>) -> Result<Vec<Transaction>, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new("SELECT * FROM transaction");
        qb.push(" WHERE ");
        qb.push("deleted_at IS NULL");
        qb.push(" AND ");
        qb.push("category_id = ").push_bind(self.id);
        if let Some(p) = pagination {
            qb.push(" LIMIT " ).push_bind(p.limit);
            qb.push(" OFFSET " ).push_bind(p.offset);
        }
        qb.build_query_as::<Transaction>().fetch_all(executor).await
    }

    // --- Relational Helper: has many ---
    #[tracing::instrument]
    pub async fn find_budgets<'c, E>(&self, executor: E, pagination: Option<Pagination>) -> Result<Vec<Budget>, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new("SELECT * FROM budget");
        qb.push(" WHERE ");
        qb.push("deleted_at IS NULL");
        qb.push(" AND ");
        qb.push("category_id = ").push_bind(self.id);
        if let Some(p) = pagination {
            qb.push(" LIMIT " ).push_bind(p.limit);
            qb.push(" OFFSET " ).push_bind(p.offset);
        }
        qb.build_query_as::<Budget>().fetch_all(executor).await
    }

    // --- Relational Helper: has many ---
    #[tracing::instrument]
    pub async fn find_recurringtransactions<'c, E>(&self, executor: E, pagination: Option<Pagination>) -> Result<Vec<RecurringTransaction>, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new("SELECT * FROM recurringtransaction");
        qb.push(" WHERE ");
        qb.push("deleted_at IS NULL");
        qb.push(" AND ");
        qb.push("category_id = ").push_bind(self.id);
        if let Some(p) = pagination {
            qb.push(" LIMIT " ).push_bind(p.limit);
            qb.push(" OFFSET " ).push_bind(p.offset);
        }
        qb.build_query_as::<RecurringTransaction>().fetch_all(executor).await
    }

    // --- Eager Loading Helper ---
    #[tracing::instrument]
    pub async fn find_by_ids<'c, E>(executor: E, ids: &[uuid::Uuid]) -> Result<Vec<Category>, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        sqlx::query_as("SELECT * FROM category WHERE id = ANY($1) AND deleted_at IS NULL")
            .bind(ids)
            .fetch_all(executor)
            .await
    }

}

impl Goal {
    #[tracing::instrument]
    pub async fn create<'c, E>(executor: E, item: &GoalNew, user_id: uuid::Uuid) -> Result<Goal, GoalCreateError>
    where
        E: PgExecutor<'c>,
    {
        let res = sqlx::query_as::<_, Goal>("INSERT INTO goal (amount, description, icon, name, target_amount, target_date, user_id) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *")
            .bind(&item.amount)
            .bind(&item.description)
            .bind(&item.icon)
            .bind(&item.name)
            .bind(&item.target_amount)
            .bind(&item.target_date)
            .bind(&user_id)
            .fetch_one(executor)
            .await;
        match res {
            Ok(v) => Ok(v),
            Err(e) => {
                if let sqlx::Error::Database(db_err) = &e {
                    if let Some(c) = db_err.constraint() {
                        if c == "goal_name_user_unique" { return Err(GoalCreateError::GoalNameUserUnique); }
                        if c == "user" { return Err(GoalCreateError::UserFk); }
                    }
                }
                Err(GoalCreateError::Database(e))
            }
        }
    }

    #[tracing::instrument]
    pub async fn find<'c, E>(executor: E, id: uuid::Uuid, user_id: uuid::Uuid) -> Result<Goal, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        sqlx::query_as::<_, Goal>("SELECT * FROM goal WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL")
            .bind(id)
            .bind(user_id)
            .fetch_one(executor)
            .await
    }

    #[tracing::instrument]
    pub async fn update<'c, E>(executor: E, id: uuid::Uuid, user_id: uuid::Uuid, item: &GoalUpdate) -> Result<Goal, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new("UPDATE goal SET " );
        let mut has_updates = false;
        let mut separated = qb.separated(", ");
        if let Some(value) = &item.amount { separated.push("amount = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.deleted_at { separated.push("deleted_at = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.description { separated.push("description = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.icon { separated.push("icon = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.name { separated.push("name = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.target_amount { separated.push("target_amount = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.target_date { separated.push("target_date = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.user_id { separated.push("user_id = ").push_bind(value); has_updates = true; }
        if !has_updates {
            // Can't call Self::find with a generic executor easily, so we query directly
            return sqlx::query_as("SELECT * FROM goal WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL")
                .bind(id).bind(user_id).fetch_one(executor).await;
        }
        qb.push(" WHERE id = " ).push_bind(id).push(" AND user_id = ").push_bind(user_id).push(" RETURNING *");
        let query = qb.build_query_as::<Goal>();
        query.fetch_one(executor).await
    }

    #[tracing::instrument]
    pub async fn delete<'c, E>(executor: E, id: uuid::Uuid, user_id: uuid::Uuid) -> Result<u64, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let res = sqlx::query("UPDATE goal SET deleted_at = now() WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(executor)
            .await?;
        Ok(res.rows_affected())
    }

    #[tracing::instrument]
    pub async fn list<'c, E>(executor: E, user_id: uuid::Uuid, pagination: Option<Pagination>) -> Result<Vec<Goal>, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new("SELECT * FROM goal");
        qb.push(" WHERE deleted_at IS NULL AND user_id = ").push_bind(user_id);
        if let Some(p) = pagination {
            qb.push(" LIMIT " ).push_bind(p.limit);
            qb.push(" OFFSET " ).push_bind(p.offset);
        }
        qb.build_query_as::<Goal>().fetch_all(executor).await
    }

    // --- Eager Loading Helper ---
    #[tracing::instrument]
    pub async fn find_by_ids<'c, E>(executor: E, ids: &[uuid::Uuid]) -> Result<Vec<Goal>, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        sqlx::query_as("SELECT * FROM goal WHERE id = ANY($1) AND deleted_at IS NULL")
            .bind(ids)
            .fetch_all(executor)
            .await
    }

}

impl RecurringTransaction {
    #[tracing::instrument]
    pub async fn create<'c, E>(executor: E, item: &RecurringTransactionNew, user_id: uuid::Uuid) -> Result<RecurringTransaction, RecurringTransactionCreateError>
    where
        E: PgExecutor<'c>,
    {
        let res = sqlx::query_as::<_, RecurringTransaction>("INSERT INTO recurringtransaction (account_id, amount, category_id, description, end_date, frequency, from_account_id, name, next_date, notes, start_date, to_account_id, type, user_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14) RETURNING *")
            .bind(&item.account_id)
            .bind(&item.amount)
            .bind(&item.category_id)
            .bind(&item.description)
            .bind(&item.end_date)
            .bind(&item.frequency)
            .bind(&item.from_account_id)
            .bind(&item.name)
            .bind(&item.next_date)
            .bind(&item.notes)
            .bind(&item.start_date)
            .bind(&item.to_account_id)
            .bind(&item.r#type)
            .bind(&user_id)
            .fetch_one(executor)
            .await;
        match res {
            Ok(v) => Ok(v),
            Err(e) => {
                if let sqlx::Error::Database(db_err) = &e {
                    if let Some(c) = db_err.constraint() {
                        if c == "recurring_transaction_name_user_unique" { return Err(RecurringTransactionCreateError::RecurringTransactionNameUserUnique); }
                        if c == "account" { return Err(RecurringTransactionCreateError::AccountFk); }
                        if c == "category" { return Err(RecurringTransactionCreateError::CategoryFk); }
                        if c == "from_account" { return Err(RecurringTransactionCreateError::FromAccountFk); }
                        if c == "to_account" { return Err(RecurringTransactionCreateError::ToAccountFk); }
                        if c == "user" { return Err(RecurringTransactionCreateError::UserFk); }
                    }
                }
                Err(RecurringTransactionCreateError::Database(e))
            }
        }
    }

    #[tracing::instrument]
    pub async fn find<'c, E>(executor: E, id: uuid::Uuid, user_id: uuid::Uuid) -> Result<RecurringTransaction, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        sqlx::query_as::<_, RecurringTransaction>("SELECT * FROM recurringtransaction WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL")
            .bind(id)
            .bind(user_id)
            .fetch_one(executor)
            .await
    }

    #[tracing::instrument]
    pub async fn update<'c, E>(executor: E, id: uuid::Uuid, user_id: uuid::Uuid, item: &RecurringTransactionUpdate) -> Result<RecurringTransaction, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new("UPDATE recurringtransaction SET " );
        let mut has_updates = false;
        let mut separated = qb.separated(", ");
        if let Some(value) = &item.account_id { separated.push("account_id = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.amount { separated.push("amount = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.category_id { separated.push("category_id = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.deleted_at { separated.push("deleted_at = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.description { separated.push("description = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.end_date { separated.push("end_date = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.frequency { separated.push("frequency = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.from_account_id { separated.push("from_account_id = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.name { separated.push("name = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.next_date { separated.push("next_date = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.notes { separated.push("notes = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.start_date { separated.push("start_date = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.to_account_id { separated.push("to_account_id = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.r#type { separated.push("type = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.user_id { separated.push("user_id = ").push_bind(value); has_updates = true; }
        if !has_updates {
            // Can't call Self::find with a generic executor easily, so we query directly
            return sqlx::query_as("SELECT * FROM recurringtransaction WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL")
                .bind(id).bind(user_id).fetch_one(executor).await;
        }
        qb.push(" WHERE id = " ).push_bind(id).push(" AND user_id = ").push_bind(user_id).push(" RETURNING *");
        let query = qb.build_query_as::<RecurringTransaction>();
        query.fetch_one(executor).await
    }

    #[tracing::instrument]
    pub async fn delete<'c, E>(executor: E, id: uuid::Uuid, user_id: uuid::Uuid) -> Result<u64, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let res = sqlx::query("UPDATE recurringtransaction SET deleted_at = now() WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(executor)
            .await?;
        Ok(res.rows_affected())
    }

    #[tracing::instrument]
    pub async fn list<'c, E>(executor: E, user_id: uuid::Uuid, pagination: Option<Pagination>) -> Result<Vec<RecurringTransaction>, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new("SELECT * FROM recurringtransaction");
        qb.push(" WHERE deleted_at IS NULL AND user_id = ").push_bind(user_id);
        if let Some(p) = pagination {
            qb.push(" LIMIT " ).push_bind(p.limit);
            qb.push(" OFFSET " ).push_bind(p.offset);
        }
        qb.build_query_as::<RecurringTransaction>().fetch_all(executor).await
    }

    // --- Eager Loading Helper ---
    #[tracing::instrument]
    pub async fn find_by_ids<'c, E>(executor: E, ids: &[uuid::Uuid]) -> Result<Vec<RecurringTransaction>, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        sqlx::query_as("SELECT * FROM recurringtransaction WHERE id = ANY($1) AND deleted_at IS NULL")
            .bind(ids)
            .fetch_all(executor)
            .await
    }

}

impl Transaction {
    #[tracing::instrument]
    pub async fn create<'c, E>(executor: E, item: &TransactionNew, user_id: uuid::Uuid) -> Result<Transaction, TransactionCreateError>
    where
        E: PgExecutor<'c>,
    {
        let res = sqlx::query_as::<_, Transaction>("INSERT INTO transaction (account_id, amount, category_id, description, from_account_id, notes, receipt, to_account_id, type, user_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING *")
            .bind(&item.account_id)
            .bind(&item.amount)
            .bind(&item.category_id)
            .bind(&item.description)
            .bind(&item.from_account_id)
            .bind(&item.notes)
            .bind(&item.receipt)
            .bind(&item.to_account_id)
            .bind(&item.r#type)
            .bind(&user_id)
            .fetch_one(executor)
            .await;
        match res {
            Ok(v) => Ok(v),
            Err(e) => {
                if let sqlx::Error::Database(db_err) = &e {
                    if let Some(c) = db_err.constraint() {
                        if c == "account" { return Err(TransactionCreateError::AccountFk); }
                        if c == "category" { return Err(TransactionCreateError::CategoryFk); }
                        if c == "from_account" { return Err(TransactionCreateError::FromAccountFk); }
                        if c == "to_account" { return Err(TransactionCreateError::ToAccountFk); }
                        if c == "user" { return Err(TransactionCreateError::UserFk); }
                    }
                }
                Err(TransactionCreateError::Database(e))
            }
        }
    }

    #[tracing::instrument]
    pub async fn find<'c, E>(executor: E, id: uuid::Uuid, user_id: uuid::Uuid) -> Result<Transaction, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        sqlx::query_as::<_, Transaction>("SELECT * FROM transaction WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL")
            .bind(id)
            .bind(user_id)
            .fetch_one(executor)
            .await
    }

    #[tracing::instrument]
    pub async fn update<'c, E>(executor: E, id: uuid::Uuid, user_id: uuid::Uuid, item: &TransactionUpdate) -> Result<Transaction, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new("UPDATE transaction SET " );
        let mut has_updates = false;
        let mut separated = qb.separated(", ");
        if let Some(value) = &item.account_id { separated.push("account_id = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.amount { separated.push("amount = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.category_id { separated.push("category_id = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.deleted_at { separated.push("deleted_at = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.description { separated.push("description = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.from_account_id { separated.push("from_account_id = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.notes { separated.push("notes = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.receipt { separated.push("receipt = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.to_account_id { separated.push("to_account_id = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.r#type { separated.push("type = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.user_id { separated.push("user_id = ").push_bind(value); has_updates = true; }
        if !has_updates {
            // Can't call Self::find with a generic executor easily, so we query directly
            return sqlx::query_as("SELECT * FROM transaction WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL")
                .bind(id).bind(user_id).fetch_one(executor).await;
        }
        qb.push(" WHERE id = " ).push_bind(id).push(" AND user_id = ").push_bind(user_id).push(" RETURNING *");
        let query = qb.build_query_as::<Transaction>();
        query.fetch_one(executor).await
    }

    #[tracing::instrument]
    pub async fn delete<'c, E>(executor: E, id: uuid::Uuid, user_id: uuid::Uuid) -> Result<u64, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let res = sqlx::query("UPDATE transaction SET deleted_at = now() WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(executor)
            .await?;
        Ok(res.rows_affected())
    }

    #[tracing::instrument]
    pub async fn list<'c, E>(executor: E, user_id: uuid::Uuid, pagination: Option<Pagination>) -> Result<Vec<Transaction>, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new("SELECT * FROM transaction");
        qb.push(" WHERE deleted_at IS NULL AND user_id = ").push_bind(user_id);
        if let Some(p) = pagination {
            qb.push(" LIMIT " ).push_bind(p.limit);
            qb.push(" OFFSET " ).push_bind(p.offset);
        }
        qb.build_query_as::<Transaction>().fetch_all(executor).await
    }

    // --- Eager Loading Helper ---
    #[tracing::instrument]
    pub async fn find_by_ids<'c, E>(executor: E, ids: &[uuid::Uuid]) -> Result<Vec<Transaction>, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        sqlx::query_as("SELECT * FROM transaction WHERE id = ANY($1) AND deleted_at IS NULL")
            .bind(ids)
            .fetch_all(executor)
            .await
    }

}

impl User {
    #[tracing::instrument]
    pub async fn create<'c, E>(executor: E, item: &UserNew) -> Result<User, UserCreateError>
    where
        E: PgExecutor<'c>,
    {
        let res = sqlx::query_as::<_, User>("INSERT INTO user (email, first_name, last_name, password_hash, profile_pic) VALUES ($1, $2, $3, $4, $5) RETURNING *")
            .bind(&item.email)
            .bind(&item.first_name)
            .bind(&item.last_name)
            .bind(&{
                use argon2::{Argon2, PasswordHasher};
                use password_hash::{rand_core::OsRng, SaltString};
                let salt = SaltString::generate(&mut OsRng);
                let argon2 = Argon2::default();
                argon2.hash_password(item.password.as_bytes(), &salt)
                    .map_err(|_| sqlx::Error::Protocol("Password hashing failed".into()))?
                    .to_string()
            })
            .bind(&item.profile_pic)
            .fetch_one(executor)
            .await;
        match res {
            Ok(v) => Ok(v),
            Err(e) => {
                if let sqlx::Error::Database(db_err) = &e {
                    if let Some(c) = db_err.constraint() {
                        if c == "user_email_unique" { return Err(UserCreateError::UserEmailUnique); }
                    }
                }
                Err(UserCreateError::Database(e))
            }
        }
    }

    #[tracing::instrument]
    pub async fn find<'c, E>(executor: E, id: uuid::Uuid) -> Result<User, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        sqlx::query_as::<_, User>("SELECT * FROM user WHERE id = $1 AND deleted_at IS NULL")
            .bind(id)
            .fetch_one(executor)
            .await
    }

    #[tracing::instrument]
    pub async fn update<'c, E>(executor: E, id: uuid::Uuid, item: &UserUpdate) -> Result<User, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new("UPDATE user SET " );
        let mut has_updates = false;
        let mut separated = qb.separated(", ");
        if let Some(value) = &item.deleted_at { separated.push("deleted_at = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.email { separated.push("email = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.first_name { separated.push("first_name = ").push_bind(value); has_updates = true; }
        if let Some(value) = &item.last_name { separated.push("last_name = ").push_bind(value); has_updates = true; }
        if let Some(password) = &item.password {
            use argon2::{Argon2, PasswordHasher};
            use password_hash::{rand_core::OsRng, SaltString};
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            let hashed = argon2.hash_password(password.as_bytes(), &salt)
                .map_err(|_| sqlx::Error::Protocol("Password hashing failed".into()))?
                .to_string();
            separated.push("password_hash = ").push_bind(hashed); has_updates = true;
        }
        if let Some(value) = &item.profile_pic { separated.push("profile_pic = ").push_bind(value); has_updates = true; }
        if !has_updates {
            return sqlx::query_as("SELECT * FROM user WHERE id = $1 AND deleted_at IS NULL")
                .bind(id).fetch_one(executor).await;
        }
        qb.push(" WHERE id = " ).push_bind(id).push(" RETURNING *");
        let query = qb.build_query_as::<User>();
        query.fetch_one(executor).await
    }

    #[tracing::instrument]
    pub async fn delete<'c, E>(executor: E, id: uuid::Uuid) -> Result<u64, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let res = sqlx::query("UPDATE user SET deleted_at = now() WHERE id = $1")
            .bind(id)
            .execute(executor)
            .await?;
        Ok(res.rows_affected())
    }

    #[tracing::instrument]
    pub async fn list<'c, E>(executor: E, pagination: Option<Pagination>) -> Result<Vec<User>, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new("SELECT * FROM user");
        qb.push(" WHERE deleted_at IS NULL");
        if let Some(p) = pagination {
            qb.push(" LIMIT " ).push_bind(p.limit);
            qb.push(" OFFSET " ).push_bind(p.offset);
        }
        qb.build_query_as::<User>().fetch_all(executor).await
    }

    // --- Relational Helper: has many ---
    #[tracing::instrument]
    pub async fn find_transactions<'c, E>(&self, executor: E, pagination: Option<Pagination>) -> Result<Vec<Transaction>, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new("SELECT * FROM transaction");
        qb.push(" WHERE ");
        qb.push("deleted_at IS NULL");
        qb.push(" AND ");
        qb.push("user_id = ").push_bind(self.id);
        if let Some(p) = pagination {
            qb.push(" LIMIT " ).push_bind(p.limit);
            qb.push(" OFFSET " ).push_bind(p.offset);
        }
        qb.build_query_as::<Transaction>().fetch_all(executor).await
    }

    // --- Relational Helper: has many ---
    #[tracing::instrument]
    pub async fn find_budgets<'c, E>(&self, executor: E, pagination: Option<Pagination>) -> Result<Vec<Budget>, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new("SELECT * FROM budget");
        qb.push(" WHERE ");
        qb.push("deleted_at IS NULL");
        qb.push(" AND ");
        qb.push("user_id = ").push_bind(self.id);
        if let Some(p) = pagination {
            qb.push(" LIMIT " ).push_bind(p.limit);
            qb.push(" OFFSET " ).push_bind(p.offset);
        }
        qb.build_query_as::<Budget>().fetch_all(executor).await
    }

    // --- Relational Helper: has many ---
    #[tracing::instrument]
    pub async fn find_goals<'c, E>(&self, executor: E, pagination: Option<Pagination>) -> Result<Vec<Goal>, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new("SELECT * FROM goal");
        qb.push(" WHERE ");
        qb.push("deleted_at IS NULL");
        qb.push(" AND ");
        qb.push("user_id = ").push_bind(self.id);
        if let Some(p) = pagination {
            qb.push(" LIMIT " ).push_bind(p.limit);
            qb.push(" OFFSET " ).push_bind(p.offset);
        }
        qb.build_query_as::<Goal>().fetch_all(executor).await
    }

    // --- Relational Helper: has many ---
    #[tracing::instrument]
    pub async fn find_accounts<'c, E>(&self, executor: E, pagination: Option<Pagination>) -> Result<Vec<Account>, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new("SELECT * FROM account");
        qb.push(" WHERE ");
        qb.push("deleted_at IS NULL");
        qb.push(" AND ");
        qb.push("user_id = ").push_bind(self.id);
        if let Some(p) = pagination {
            qb.push(" LIMIT " ).push_bind(p.limit);
            qb.push(" OFFSET " ).push_bind(p.offset);
        }
        qb.build_query_as::<Account>().fetch_all(executor).await
    }

    // --- Relational Helper: has many ---
    #[tracing::instrument]
    pub async fn find_categorys<'c, E>(&self, executor: E, pagination: Option<Pagination>) -> Result<Vec<Category>, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new("SELECT * FROM category");
        qb.push(" WHERE ");
        qb.push("deleted_at IS NULL");
        qb.push(" AND ");
        qb.push("user_id = ").push_bind(self.id);
        if let Some(p) = pagination {
            qb.push(" LIMIT " ).push_bind(p.limit);
            qb.push(" OFFSET " ).push_bind(p.offset);
        }
        qb.build_query_as::<Category>().fetch_all(executor).await
    }

    // --- Relational Helper: has many ---
    #[tracing::instrument]
    pub async fn find_recurringtransactions<'c, E>(&self, executor: E, pagination: Option<Pagination>) -> Result<Vec<RecurringTransaction>, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new("SELECT * FROM recurringtransaction");
        qb.push(" WHERE ");
        qb.push("deleted_at IS NULL");
        qb.push(" AND ");
        qb.push("user_id = ").push_bind(self.id);
        if let Some(p) = pagination {
            qb.push(" LIMIT " ).push_bind(p.limit);
            qb.push(" OFFSET " ).push_bind(p.offset);
        }
        qb.build_query_as::<RecurringTransaction>().fetch_all(executor).await
    }

    // --- Eager Loading Helper ---
    #[tracing::instrument]
    pub async fn find_by_ids<'c, E>(executor: E, ids: &[uuid::Uuid]) -> Result<Vec<User>, sqlx::Error>
    where
        E: PgExecutor<'c>,
    {
        sqlx::query_as("SELECT * FROM user WHERE id = ANY($1) AND deleted_at IS NULL")
            .bind(ids)
            .fetch_all(executor)
            .await
    }

}

