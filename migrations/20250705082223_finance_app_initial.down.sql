BEGIN;
DROP TYPE accounttype;
DROP TYPE budgetperiod;
DROP TYPE goalstatus;
DROP TYPE recurringfrequency;
DROP TYPE role;
DROP TYPE transactionstatus;
DROP TYPE transactiontype;
DROP TABLE account;

DROP TABLE budget;

DROP TABLE category;

DROP TABLE goal;

DROP TABLE recurringtransaction;

DROP TABLE transaction;

DROP TABLE user;

DROP INDEX account_name_user_idx
DROP INDEX account_user_idx
DROP INDEX budget_category_idx
DROP INDEX budget_date_range_idx
DROP INDEX budget_period_idx
DROP INDEX budget_user_idx
DROP INDEX category_type_idx
DROP INDEX category_user_idx
DROP INDEX goal_status_idx
DROP INDEX goal_target_date_idx
DROP INDEX goal_user_idx
DROP INDEX recurring_transaction_account_idx
DROP INDEX recurring_transaction_category_idx
DROP INDEX recurring_transaction_frequency_idx
DROP INDEX recurring_transaction_next_date_idx
DROP INDEX recurring_transaction_user_idx
DROP INDEX transaction_account_idx
DROP INDEX transaction_category_idx
DROP INDEX transaction_date_idx
DROP INDEX transaction_type_idx
DROP INDEX transaction_user_idx
DROP INDEX user_email_unique
ALTER TABLE account DROP CONSTRAINT account_name_user_unique
ALTER TABLE budget DROP CONSTRAINT budget_name_user_unique
ALTER TABLE category DROP CONSTRAINT category_name_user_unique
ALTER TABLE goal DROP CONSTRAINT goal_name_user_unique
ALTER TABLE recurringtransaction DROP CONSTRAINT recurring_transaction_name_user_unique
ALTER TABLE account DROP CONSTRAINT account_balance_positive
ALTER TABLE account DROP CONSTRAINT account_name_length
ALTER TABLE budget DROP CONSTRAINT budget_amount_positive
ALTER TABLE budget DROP CONSTRAINT budget_date_range
ALTER TABLE budget DROP CONSTRAINT budget_name_length
ALTER TABLE category DROP CONSTRAINT category_color_format
ALTER TABLE category DROP CONSTRAINT category_name_length
ALTER TABLE goal DROP CONSTRAINT goal_color_format
ALTER TABLE goal DROP CONSTRAINT goal_current_amount_positive
ALTER TABLE goal DROP CONSTRAINT goal_current_not_exceed_target
ALTER TABLE goal DROP CONSTRAINT goal_name_length
ALTER TABLE goal DROP CONSTRAINT goal_target_amount_positive
ALTER TABLE recurringtransaction DROP CONSTRAINT recurring_transaction_amount_positive
ALTER TABLE recurringtransaction DROP CONSTRAINT recurring_transaction_date_range
ALTER TABLE recurringtransaction DROP CONSTRAINT recurring_transaction_description_length
ALTER TABLE recurringtransaction DROP CONSTRAINT recurring_transaction_name_length
ALTER TABLE recurringtransaction DROP CONSTRAINT recurring_transaction_next_date_valid
ALTER TABLE recurringtransaction DROP CONSTRAINT transfer_accounts_different
ALTER TABLE recurringtransaction DROP CONSTRAINT transfer_accounts_required
ALTER TABLE transaction DROP CONSTRAINT transaction_amount_positive
ALTER TABLE transaction DROP CONSTRAINT transaction_description_length
ALTER TABLE transaction DROP CONSTRAINT transfer_accounts_different
ALTER TABLE transaction DROP CONSTRAINT transfer_accounts_required
ALTER TABLE user DROP CONSTRAINT user_email_not_empty
ALTER TABLE user DROP CONSTRAINT user_name_length
ALTER TABLE account DROP CONSTRAINT transactions
ALTER TABLE account DROP CONSTRAINT transfers_from
ALTER TABLE account DROP CONSTRAINT transfers_to
ALTER TABLE account DROP CONSTRAINT user
ALTER TABLE budget DROP CONSTRAINT category
ALTER TABLE budget DROP CONSTRAINT user
ALTER TABLE category DROP CONSTRAINT budgets
ALTER TABLE category DROP CONSTRAINT transactions
ALTER TABLE category DROP CONSTRAINT user
ALTER TABLE goal DROP CONSTRAINT user
ALTER TABLE recurringtransaction DROP CONSTRAINT account
ALTER TABLE recurringtransaction DROP CONSTRAINT category
ALTER TABLE recurringtransaction DROP CONSTRAINT from_account
ALTER TABLE recurringtransaction DROP CONSTRAINT to_account
ALTER TABLE recurringtransaction DROP CONSTRAINT user
ALTER TABLE transaction DROP CONSTRAINT account
ALTER TABLE transaction DROP CONSTRAINT category
ALTER TABLE transaction DROP CONSTRAINT from_account
ALTER TABLE transaction DROP CONSTRAINT to_account
ALTER TABLE transaction DROP CONSTRAINT user
ALTER TABLE user DROP CONSTRAINT accounts
ALTER TABLE user DROP CONSTRAINT budgets
ALTER TABLE user DROP CONSTRAINT goals
ALTER TABLE user DROP CONSTRAINT transactions
COMMIT;
