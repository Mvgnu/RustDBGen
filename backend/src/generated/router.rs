use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use crate::generated::*;
use crate::generated::main::AppState;
use crate::generated::auth::Claims;

pub fn create_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/account", get(list_account))
        .route("/account", post(create_account))
        .route("/account/:id", get(get_account))
        .route("/account/:id", put(update_account))
        .route("/account/:id", delete(delete_account))
        .route("/budget", get(list_budget))
        .route("/budget", post(create_budget))
        .route("/budget/:id", get(get_budget))
        .route("/budget/:id", put(update_budget))
        .route("/budget/:id", delete(delete_budget))
        .route("/category", get(list_category))
        .route("/category", post(create_category))
        .route("/category/:id", get(get_category))
        .route("/category/:id", put(update_category))
        .route("/category/:id", delete(delete_category))
        .route("/goal", get(list_goal))
        .route("/goal", post(create_goal))
        .route("/goal/:id", get(get_goal))
        .route("/goal/:id", put(update_goal))
        .route("/goal/:id", delete(delete_goal))
        .route("/recurringtransaction", get(list_recurringtransaction))
        .route("/recurringtransaction", post(create_recurringtransaction))
        .route("/recurringtransaction/:id", get(get_recurringtransaction))
        .route("/recurringtransaction/:id", put(update_recurringtransaction))
        .route("/recurringtransaction/:id", delete(delete_recurringtransaction))
        .route("/transaction", get(list_transaction))
        .route("/transaction", post(create_transaction))
        .route("/transaction/:id", get(get_transaction))
        .route("/transaction/:id", put(update_transaction))
        .route("/transaction/:id", delete(delete_transaction))
        .route("/user", get(list_user))
        .route("/user", post(create_user))
        .route("/user/:id", get(get_user))
        .route("/user/:id", put(update_user))
        .route("/user/:id", delete(delete_user))
}

async fn list_account(State(state): State<Arc<AppState>>, claims: Claims, Query(pagination): Query<Option<Pagination>>) -> Result<Json<Vec<Account>>, (StatusCode, Json<Value>)> {
    let items = Account::list(&state.pool, claims.sub, pagination)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;
    Ok(Json(items))
}

async fn create_account(State(state): State<Arc<AppState>>, claims: Claims, Json(item): Json<AccountNew>) -> Result<Json<Account>, (StatusCode, Json<Value>)> {
    let item = Account::create(&state.pool, &item, claims.sub)
        .await
        .map_err(|e| {
            let status = match e {
                AccountCreateError::AccountNameUserUnique => StatusCode::CONFLICT,
                AccountCreateError::UserFk => StatusCode::BAD_REQUEST,
                AccountCreateError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(json!({"error": e.to_string()})))
        })?;
    Ok(Json(item))
}

async fn get_account(State(state): State<Arc<AppState>>, claims: Claims, Path(id): Path<uuid::Uuid>) -> Result<Json<Account>, (StatusCode, Json<Value>)> {
    let item = Account::find(&state.pool, id, claims.sub)
        .await
        .map_err(|e| {
            if let sqlx::Error::RowNotFound = e {
                (StatusCode::NOT_FOUND, Json(json!({"error": "Not found"})))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
            }
        })?;
    Ok(Json(item))
}

async fn update_account(State(state): State<Arc<AppState>>, claims: Claims, Path(id): Path<uuid::Uuid>, Json(item): Json<AccountUpdate>) -> Result<Json<Account>, (StatusCode, Json<Value>)> {
    let item = Account::update(&state.pool, id, claims.sub, &item)
        .await
        .map_err(|e| {
            if let sqlx::Error::RowNotFound = e {
                (StatusCode::NOT_FOUND, Json(json!({"error": "Not found"})))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
            }
        })?;
    Ok(Json(item))
}

async fn delete_account(State(state): State<Arc<AppState>>, claims: Claims, Path(id): Path<uuid::Uuid>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let affected = Account::delete(&state.pool, id, claims.sub)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;
    if affected == 0 {
        return Err((StatusCode::NOT_FOUND, Json(json!({"error": "Not found"}))));
    }
    Ok(Json(json!({"message": "Deleted successfully"})))
}

async fn list_budget(State(state): State<Arc<AppState>>, claims: Claims, Query(pagination): Query<Option<Pagination>>) -> Result<Json<Vec<Budget>>, (StatusCode, Json<Value>)> {
    let items = Budget::list(&state.pool, claims.sub, pagination)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;
    Ok(Json(items))
}

async fn create_budget(State(state): State<Arc<AppState>>, claims: Claims, Json(item): Json<BudgetNew>) -> Result<Json<Budget>, (StatusCode, Json<Value>)> {
    let item = Budget::create(&state.pool, &item, claims.sub)
        .await
        .map_err(|e| {
            let status = match e {
                BudgetCreateError::BudgetNameUserUnique => StatusCode::CONFLICT,
                BudgetCreateError::CategoryFk => StatusCode::BAD_REQUEST,
                BudgetCreateError::UserFk => StatusCode::BAD_REQUEST,
                BudgetCreateError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(json!({"error": e.to_string()})))
        })?;
    Ok(Json(item))
}

async fn get_budget(State(state): State<Arc<AppState>>, claims: Claims, Path(id): Path<uuid::Uuid>) -> Result<Json<Budget>, (StatusCode, Json<Value>)> {
    let item = Budget::find(&state.pool, id, claims.sub)
        .await
        .map_err(|e| {
            if let sqlx::Error::RowNotFound = e {
                (StatusCode::NOT_FOUND, Json(json!({"error": "Not found"})))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
            }
        })?;
    Ok(Json(item))
}

async fn update_budget(State(state): State<Arc<AppState>>, claims: Claims, Path(id): Path<uuid::Uuid>, Json(item): Json<BudgetUpdate>) -> Result<Json<Budget>, (StatusCode, Json<Value>)> {
    let item = Budget::update(&state.pool, id, claims.sub, &item)
        .await
        .map_err(|e| {
            if let sqlx::Error::RowNotFound = e {
                (StatusCode::NOT_FOUND, Json(json!({"error": "Not found"})))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
            }
        })?;
    Ok(Json(item))
}

async fn delete_budget(State(state): State<Arc<AppState>>, claims: Claims, Path(id): Path<uuid::Uuid>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let affected = Budget::delete(&state.pool, id, claims.sub)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;
    if affected == 0 {
        return Err((StatusCode::NOT_FOUND, Json(json!({"error": "Not found"}))));
    }
    Ok(Json(json!({"message": "Deleted successfully"})))
}

async fn list_category(State(state): State<Arc<AppState>>, Query(pagination): Query<Option<Pagination>>) -> Result<Json<Vec<Category>>, (StatusCode, Json<Value>)> {
    let items = Category::list(&state.pool, pagination)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;
    Ok(Json(items))
}

async fn create_category(State(state): State<Arc<AppState>>, Json(item): Json<CategoryNew>) -> Result<Json<Category>, (StatusCode, Json<Value>)> {
    let item = Category::create(&state.pool, &item)
        .await
        .map_err(|e| {
            let status = match e {
                CategoryCreateError::CategoryNameUserUnique => StatusCode::CONFLICT,
                CategoryCreateError::UserFk => StatusCode::BAD_REQUEST,
                CategoryCreateError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(json!({"error": e.to_string()})))
        })?;
    Ok(Json(item))
}

async fn get_category(State(state): State<Arc<AppState>>, Path(id): Path<uuid::Uuid>) -> Result<Json<Category>, (StatusCode, Json<Value>)> {
    let item = Category::find(&state.pool, id)
        .await
        .map_err(|e| {
            if let sqlx::Error::RowNotFound = e {
                (StatusCode::NOT_FOUND, Json(json!({"error": "Not found"})))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
            }
        })?;
    Ok(Json(item))
}

async fn update_category(State(state): State<Arc<AppState>>, Path(id): Path<uuid::Uuid>, Json(item): Json<CategoryUpdate>) -> Result<Json<Category>, (StatusCode, Json<Value>)> {
    let item = Category::update(&state.pool, id, &item)
        .await
        .map_err(|e| {
            if let sqlx::Error::RowNotFound = e {
                (StatusCode::NOT_FOUND, Json(json!({"error": "Not found"})))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
            }
        })?;
    Ok(Json(item))
}

async fn delete_category(State(state): State<Arc<AppState>>, Path(id): Path<uuid::Uuid>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let affected = Category::delete(&state.pool, id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;
    if affected == 0 {
        return Err((StatusCode::NOT_FOUND, Json(json!({"error": "Not found"}))));
    }
    Ok(Json(json!({"message": "Deleted successfully"})))
}

async fn list_goal(State(state): State<Arc<AppState>>, claims: Claims, Query(pagination): Query<Option<Pagination>>) -> Result<Json<Vec<Goal>>, (StatusCode, Json<Value>)> {
    let items = Goal::list(&state.pool, claims.sub, pagination)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;
    Ok(Json(items))
}

async fn create_goal(State(state): State<Arc<AppState>>, claims: Claims, Json(item): Json<GoalNew>) -> Result<Json<Goal>, (StatusCode, Json<Value>)> {
    let item = Goal::create(&state.pool, &item, claims.sub)
        .await
        .map_err(|e| {
            let status = match e {
                GoalCreateError::GoalNameUserUnique => StatusCode::CONFLICT,
                GoalCreateError::UserFk => StatusCode::BAD_REQUEST,
                GoalCreateError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(json!({"error": e.to_string()})))
        })?;
    Ok(Json(item))
}

async fn get_goal(State(state): State<Arc<AppState>>, claims: Claims, Path(id): Path<uuid::Uuid>) -> Result<Json<Goal>, (StatusCode, Json<Value>)> {
    let item = Goal::find(&state.pool, id, claims.sub)
        .await
        .map_err(|e| {
            if let sqlx::Error::RowNotFound = e {
                (StatusCode::NOT_FOUND, Json(json!({"error": "Not found"})))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
            }
        })?;
    Ok(Json(item))
}

async fn update_goal(State(state): State<Arc<AppState>>, claims: Claims, Path(id): Path<uuid::Uuid>, Json(item): Json<GoalUpdate>) -> Result<Json<Goal>, (StatusCode, Json<Value>)> {
    let item = Goal::update(&state.pool, id, claims.sub, &item)
        .await
        .map_err(|e| {
            if let sqlx::Error::RowNotFound = e {
                (StatusCode::NOT_FOUND, Json(json!({"error": "Not found"})))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
            }
        })?;
    Ok(Json(item))
}

async fn delete_goal(State(state): State<Arc<AppState>>, claims: Claims, Path(id): Path<uuid::Uuid>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let affected = Goal::delete(&state.pool, id, claims.sub)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;
    if affected == 0 {
        return Err((StatusCode::NOT_FOUND, Json(json!({"error": "Not found"}))));
    }
    Ok(Json(json!({"message": "Deleted successfully"})))
}

async fn list_recurringtransaction(State(state): State<Arc<AppState>>, claims: Claims, Query(pagination): Query<Option<Pagination>>) -> Result<Json<Vec<RecurringTransaction>>, (StatusCode, Json<Value>)> {
    let items = RecurringTransaction::list(&state.pool, claims.sub, pagination)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;
    Ok(Json(items))
}

async fn create_recurringtransaction(State(state): State<Arc<AppState>>, claims: Claims, Json(item): Json<RecurringTransactionNew>) -> Result<Json<RecurringTransaction>, (StatusCode, Json<Value>)> {
    let item = RecurringTransaction::create(&state.pool, &item, claims.sub)
        .await
        .map_err(|e| {
            let status = match e {
                RecurringTransactionCreateError::RecurringTransactionNameUserUnique => StatusCode::CONFLICT,
                RecurringTransactionCreateError::AccountFk => StatusCode::BAD_REQUEST,
                RecurringTransactionCreateError::CategoryFk => StatusCode::BAD_REQUEST,
                RecurringTransactionCreateError::FromAccountFk => StatusCode::BAD_REQUEST,
                RecurringTransactionCreateError::ToAccountFk => StatusCode::BAD_REQUEST,
                RecurringTransactionCreateError::UserFk => StatusCode::BAD_REQUEST,
                RecurringTransactionCreateError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(json!({"error": e.to_string()})))
        })?;
    Ok(Json(item))
}

async fn get_recurringtransaction(State(state): State<Arc<AppState>>, claims: Claims, Path(id): Path<uuid::Uuid>) -> Result<Json<RecurringTransaction>, (StatusCode, Json<Value>)> {
    let item = RecurringTransaction::find(&state.pool, id, claims.sub)
        .await
        .map_err(|e| {
            if let sqlx::Error::RowNotFound = e {
                (StatusCode::NOT_FOUND, Json(json!({"error": "Not found"})))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
            }
        })?;
    Ok(Json(item))
}

async fn update_recurringtransaction(State(state): State<Arc<AppState>>, claims: Claims, Path(id): Path<uuid::Uuid>, Json(item): Json<RecurringTransactionUpdate>) -> Result<Json<RecurringTransaction>, (StatusCode, Json<Value>)> {
    let item = RecurringTransaction::update(&state.pool, id, claims.sub, &item)
        .await
        .map_err(|e| {
            if let sqlx::Error::RowNotFound = e {
                (StatusCode::NOT_FOUND, Json(json!({"error": "Not found"})))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
            }
        })?;
    Ok(Json(item))
}

async fn delete_recurringtransaction(State(state): State<Arc<AppState>>, claims: Claims, Path(id): Path<uuid::Uuid>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let affected = RecurringTransaction::delete(&state.pool, id, claims.sub)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;
    if affected == 0 {
        return Err((StatusCode::NOT_FOUND, Json(json!({"error": "Not found"}))));
    }
    Ok(Json(json!({"message": "Deleted successfully"})))
}

async fn list_transaction(State(state): State<Arc<AppState>>, claims: Claims, Query(pagination): Query<Option<Pagination>>) -> Result<Json<Vec<Transaction>>, (StatusCode, Json<Value>)> {
    let items = Transaction::list(&state.pool, claims.sub, pagination)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;
    Ok(Json(items))
}

async fn create_transaction(State(state): State<Arc<AppState>>, claims: Claims, Json(item): Json<TransactionNew>) -> Result<Json<Transaction>, (StatusCode, Json<Value>)> {
    let item = Transaction::create(&state.pool, &item, claims.sub)
        .await
        .map_err(|e| {
            let status = match e {
                TransactionCreateError::AccountFk => StatusCode::BAD_REQUEST,
                TransactionCreateError::CategoryFk => StatusCode::BAD_REQUEST,
                TransactionCreateError::FromAccountFk => StatusCode::BAD_REQUEST,
                TransactionCreateError::ToAccountFk => StatusCode::BAD_REQUEST,
                TransactionCreateError::UserFk => StatusCode::BAD_REQUEST,
                TransactionCreateError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(json!({"error": e.to_string()})))
        })?;
    Ok(Json(item))
}

async fn get_transaction(State(state): State<Arc<AppState>>, claims: Claims, Path(id): Path<uuid::Uuid>) -> Result<Json<Transaction>, (StatusCode, Json<Value>)> {
    let item = Transaction::find(&state.pool, id, claims.sub)
        .await
        .map_err(|e| {
            if let sqlx::Error::RowNotFound = e {
                (StatusCode::NOT_FOUND, Json(json!({"error": "Not found"})))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
            }
        })?;
    Ok(Json(item))
}

async fn update_transaction(State(state): State<Arc<AppState>>, claims: Claims, Path(id): Path<uuid::Uuid>, Json(item): Json<TransactionUpdate>) -> Result<Json<Transaction>, (StatusCode, Json<Value>)> {
    let item = Transaction::update(&state.pool, id, claims.sub, &item)
        .await
        .map_err(|e| {
            if let sqlx::Error::RowNotFound = e {
                (StatusCode::NOT_FOUND, Json(json!({"error": "Not found"})))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
            }
        })?;
    Ok(Json(item))
}

async fn delete_transaction(State(state): State<Arc<AppState>>, claims: Claims, Path(id): Path<uuid::Uuid>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let affected = Transaction::delete(&state.pool, id, claims.sub)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;
    if affected == 0 {
        return Err((StatusCode::NOT_FOUND, Json(json!({"error": "Not found"}))));
    }
    Ok(Json(json!({"message": "Deleted successfully"})))
}

async fn list_user(State(state): State<Arc<AppState>>, Query(pagination): Query<Option<Pagination>>) -> Result<Json<Vec<User>>, (StatusCode, Json<Value>)> {
    let items = User::list(&state.pool, pagination)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;
    Ok(Json(items))
}

async fn create_user(State(state): State<Arc<AppState>>, Json(item): Json<UserNew>) -> Result<Json<User>, (StatusCode, Json<Value>)> {
    let item = User::create(&state.pool, &item)
        .await
        .map_err(|e| {
            let status = match e {
                UserCreateError::UserEmailUnique => StatusCode::CONFLICT,
                UserCreateError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(json!({"error": e.to_string()})))
        })?;
    Ok(Json(item))
}

async fn get_user(State(state): State<Arc<AppState>>, Path(id): Path<uuid::Uuid>) -> Result<Json<User>, (StatusCode, Json<Value>)> {
    let item = User::find(&state.pool, id)
        .await
        .map_err(|e| {
            if let sqlx::Error::RowNotFound = e {
                (StatusCode::NOT_FOUND, Json(json!({"error": "Not found"})))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
            }
        })?;
    Ok(Json(item))
}

async fn update_user(State(state): State<Arc<AppState>>, Path(id): Path<uuid::Uuid>, Json(item): Json<UserUpdate>) -> Result<Json<User>, (StatusCode, Json<Value>)> {
    let item = User::update(&state.pool, id, &item)
        .await
        .map_err(|e| {
            if let sqlx::Error::RowNotFound = e {
                (StatusCode::NOT_FOUND, Json(json!({"error": "Not found"})))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
            }
        })?;
    Ok(Json(item))
}

async fn delete_user(State(state): State<Arc<AppState>>, Path(id): Path<uuid::Uuid>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let affected = User::delete(&state.pool, id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;
    if affected == 0 {
        return Err((StatusCode::NOT_FOUND, Json(json!({"error": "Not found"}))));
    }
    Ok(Json(json!({"message": "Deleted successfully"})))
}

