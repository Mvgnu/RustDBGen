use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "accounttype", rename_all = "lowercase")]
pub enum AccountType {
    Checking,
    Savings,
    Credit,
    Investment,
    Loan,
    Other,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "budgetperiod", rename_all = "lowercase")]
pub enum BudgetPeriod {
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "goalstatus", rename_all = "lowercase")]
pub enum GoalStatus {
    Active,
    Completed,
    Paused,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "recurringfrequency", rename_all = "lowercase")]
pub enum RecurringFrequency {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "role", rename_all = "lowercase")]
pub enum Role {
    Admin,
    Member,
    Guest,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "transactionstatus", rename_all = "lowercase")]
pub enum TransactionStatus {
    Pending,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "transactiontype", rename_all = "lowercase")]
pub enum TransactionType {
    Income,
    Expense,
    Transfer,
    Adjustment,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Account {
    pub amount: Decimal,
    pub balance: Decimal,
    pub created_at: DateTime<Utc>,
    pub currency: String,
    pub deleted_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub id: Uuid,
    pub is_active: bool,
    pub name: String,
    pub r#type: AccountType,
    pub updated_at: DateTime<Utc>,
    pub user_id: Uuid,
}

#[derive(Debug, serde::Deserialize)]
pub struct AccountNew {
    pub amount: Decimal,
    pub description: Option<String>,
    pub name: String,
    pub r#type: AccountType,
}

#[derive(Debug, serde::Deserialize, Default)]
pub struct AccountUpdate {
    pub amount: Option<Decimal>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub r#type: Option<AccountType>,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Budget {
    pub amount: Decimal,
    pub category_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub currency: String,
    pub deleted_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub end_date: Option<DateTime<Utc>>,
    pub id: Uuid,
    pub is_active: bool,
    pub name: String,
    pub period: BudgetPeriod,
    pub start_date: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user_id: Uuid,
}

#[derive(Debug, serde::Deserialize)]
pub struct BudgetNew {
    pub amount: Decimal,
    pub category_id: Option<Uuid>,
    pub description: Option<String>,
    pub end_date: Option<DateTime<Utc>>,
    pub name: String,
    pub period: BudgetPeriod,
    pub start_date: DateTime<Utc>,
}

#[derive(Debug, serde::Deserialize, Default)]
pub struct BudgetUpdate {
    pub amount: Option<Decimal>,
    pub category_id: Option<Uuid>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub end_date: Option<DateTime<Utc>>,
    pub name: Option<String>,
    pub period: Option<BudgetPeriod>,
    pub start_date: Option<DateTime<Utc>>,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Category {
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub id: Uuid,
    pub is_default: bool,
    pub name: String,
    pub r#type: TransactionType,
    pub updated_at: DateTime<Utc>,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, serde::Deserialize)]
pub struct CategoryNew {
    pub description: Option<String>,
    pub icon: Option<String>,
    pub name: String,
    pub r#type: TransactionType,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, serde::Deserialize, Default)]
pub struct CategoryUpdate {
    pub deleted_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub name: Option<String>,
    pub r#type: Option<TransactionType>,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Goal {
    pub amount: Decimal,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
    pub currency: String,
    pub current_amount: Decimal,
    pub deleted_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub id: Uuid,
    pub name: String,
    pub status: GoalStatus,
    pub target_amount: Decimal,
    pub target_date: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
    pub user_id: Uuid,
}

#[derive(Debug, serde::Deserialize)]
pub struct GoalNew {
    pub amount: Decimal,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub name: String,
    pub target_amount: Decimal,
    pub target_date: Option<DateTime<Utc>>,
}

#[derive(Debug, serde::Deserialize, Default)]
pub struct GoalUpdate {
    pub amount: Option<Decimal>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub name: Option<String>,
    pub target_amount: Option<Decimal>,
    pub target_date: Option<DateTime<Utc>>,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct RecurringTransaction {
    pub account_id: Uuid,
    pub amount: Decimal,
    pub category_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub currency: String,
    pub deleted_at: Option<DateTime<Utc>>,
    pub description: String,
    pub end_date: Option<DateTime<Utc>>,
    pub frequency: RecurringFrequency,
    pub from_account_id: Option<Uuid>,
    pub id: Uuid,
    pub is_active: bool,
    pub name: String,
    pub next_date: DateTime<Utc>,
    pub notes: Option<String>,
    pub start_date: DateTime<Utc>,
    pub to_account_id: Option<Uuid>,
    pub r#type: TransactionType,
    pub updated_at: DateTime<Utc>,
    pub user_id: Uuid,
}

#[derive(Debug, serde::Deserialize)]
pub struct RecurringTransactionNew {
    pub account_id: Uuid,
    pub amount: Decimal,
    pub category_id: Option<Uuid>,
    pub description: String,
    pub end_date: Option<DateTime<Utc>>,
    pub frequency: RecurringFrequency,
    pub from_account_id: Option<Uuid>,
    pub name: String,
    pub next_date: DateTime<Utc>,
    pub notes: Option<String>,
    pub start_date: DateTime<Utc>,
    pub to_account_id: Option<Uuid>,
    pub r#type: TransactionType,
}

#[derive(Debug, serde::Deserialize, Default)]
pub struct RecurringTransactionUpdate {
    pub account_id: Option<Uuid>,
    pub amount: Option<Decimal>,
    pub category_id: Option<Uuid>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub end_date: Option<DateTime<Utc>>,
    pub frequency: Option<RecurringFrequency>,
    pub from_account_id: Option<Uuid>,
    pub name: Option<String>,
    pub next_date: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub to_account_id: Option<Uuid>,
    pub r#type: Option<TransactionType>,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Transaction {
    pub account_id: Uuid,
    pub amount: Decimal,
    pub category_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub currency: String,
    pub date: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub description: String,
    pub from_account_id: Option<Uuid>,
    pub id: Uuid,
    pub notes: Option<String>,
    pub receipt: String,
    pub status: TransactionStatus,
    pub to_account_id: Option<Uuid>,
    pub r#type: TransactionType,
    pub updated_at: DateTime<Utc>,
    pub user_id: Uuid,
}

#[derive(Debug, serde::Deserialize)]
pub struct TransactionNew {
    pub account_id: Uuid,
    pub amount: Decimal,
    pub category_id: Option<Uuid>,
    pub description: String,
    pub from_account_id: Option<Uuid>,
    pub notes: Option<String>,
    pub receipt: String,
    pub to_account_id: Option<Uuid>,
    pub r#type: TransactionType,
}

#[derive(Debug, serde::Deserialize, Default)]
pub struct TransactionUpdate {
    pub account_id: Option<Uuid>,
    pub amount: Option<Decimal>,
    pub category_id: Option<Uuid>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub from_account_id: Option<Uuid>,
    pub notes: Option<String>,
    pub receipt: Option<String>,
    pub to_account_id: Option<Uuid>,
    pub r#type: Option<TransactionType>,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct User {
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub email: String,
    pub first_name: Option<String>,
    pub id: Uuid,
    pub is_active: bool,
    pub last_name: Option<String>,
    pub password_hash: String,
    pub profile_pic: String,
    pub role: Role,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, serde::Deserialize)]
pub struct UserNew {
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub password: String,
    pub profile_pic: String,
}

#[derive(Debug, serde::Deserialize, Default)]
pub struct UserUpdate {
    pub deleted_at: Option<DateTime<Utc>>,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub password: Option<String>,
    pub profile_pic: Option<String>,
}

#[derive(Debug, Error)]
pub enum AccountCreateError {
    #[error("unique constraint `account_name_user_unique` violated")]
    AccountNameUserUnique,
    #[error("foreign key `user` violation")]
    UserFk,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum BudgetCreateError {
    #[error("unique constraint `budget_name_user_unique` violated")]
    BudgetNameUserUnique,
    #[error("foreign key `category` violation")]
    CategoryFk,
    #[error("foreign key `user` violation")]
    UserFk,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum CategoryCreateError {
    #[error("unique constraint `category_name_user_unique` violated")]
    CategoryNameUserUnique,
    #[error("foreign key `user` violation")]
    UserFk,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum GoalCreateError {
    #[error("unique constraint `goal_name_user_unique` violated")]
    GoalNameUserUnique,
    #[error("foreign key `user` violation")]
    UserFk,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum RecurringTransactionCreateError {
    #[error("unique constraint `recurring_transaction_name_user_unique` violated")]
    RecurringTransactionNameUserUnique,
    #[error("foreign key `account` violation")]
    AccountFk,
    #[error("foreign key `category` violation")]
    CategoryFk,
    #[error("foreign key `from_account` violation")]
    FromAccountFk,
    #[error("foreign key `to_account` violation")]
    ToAccountFk,
    #[error("foreign key `user` violation")]
    UserFk,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum TransactionCreateError {
    #[error("foreign key `account` violation")]
    AccountFk,
    #[error("foreign key `category` violation")]
    CategoryFk,
    #[error("foreign key `from_account` violation")]
    FromAccountFk,
    #[error("foreign key `to_account` violation")]
    ToAccountFk,
    #[error("foreign key `user` violation")]
    UserFk,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum UserCreateError {
    #[error("unique constraint `user_email_unique` violated")]
    UserEmailUnique,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

