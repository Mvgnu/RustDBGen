BEGIN;
CREATE TYPE accounttype AS ENUM ('checking', 'savings', 'credit', 'investment', 'loan', 'other');
CREATE TYPE budgetperiod AS ENUM ('weekly', 'monthly', 'quarterly', 'yearly');
CREATE TYPE goalstatus AS ENUM ('active', 'completed', 'paused', 'cancelled');
CREATE TYPE recurringfrequency AS ENUM ('daily', 'weekly', 'monthly', 'quarterly', 'yearly');
CREATE TYPE role AS ENUM ('admin', 'member', 'guest');
CREATE TYPE transactionstatus AS ENUM ('pending', 'completed', 'failed', 'cancelled');
CREATE TYPE transactiontype AS ENUM ('income', 'expense', 'transfer', 'adjustment');
CREATE TABLE account (
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    is_active BOOLEAN NOT NULL DEFAULT true,
    balance DECIMAL(15,2) NOT NULL DEFAULT 0.00,
    type account_type NOT NULL,
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    amount DECIMAL(15,2) NOT NULL,
    description TEXT,
    deleted_at TIMESTAMPTZ,
    user_id UUID NOT NULL
);

CREATE TABLE budget (
    amount DECIMAL(15,2) NOT NULL,
    end_date TIMESTAMPTZ,
    category_id UUID,
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    start_date TIMESTAMPTZ NOT NULL,
    period budget_period NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    user_id UUID NOT NULL,
    deleted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    description TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE category (
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    deleted_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    type transaction_type NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT false,
    color VARCHAR(7) DEFAULT '#000000',
    icon VARCHAR(50),
    user_id UUID,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE goal (
    name VARCHAR(100) NOT NULL,
    target_date TIMESTAMPTZ,
    description TEXT,
    status goal_status NOT NULL DEFAULT 'Active',
    color VARCHAR(7) DEFAULT '#4CAF50',
    current_amount DECIMAL(15,2) NOT NULL DEFAULT 0.00,
    icon VARCHAR(50),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    user_id UUID NOT NULL,
    deleted_at TIMESTAMPTZ,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    amount DECIMAL(15,2) NOT NULL,
    target_amount DECIMAL(15,2) NOT NULL,
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid()
);

CREATE TABLE recurringtransaction (
    start_date TIMESTAMPTZ NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    account_id UUID NOT NULL,
    end_date TIMESTAMPTZ,
    notes TEXT,
    category_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    type transaction_type NOT NULL,
    from_account_id UUID,
    description TEXT NOT NULL,
    name VARCHAR(100) NOT NULL,
    frequency recurring_frequency NOT NULL,
    next_date TIMESTAMPTZ NOT NULL,
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    amount DECIMAL(15,2) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    to_account_id UUID,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    user_id UUID NOT NULL
);

CREATE TABLE transaction (
    type transaction_type NOT NULL,
    amount DECIMAL(15,2) NOT NULL,
    status transaction_status NOT NULL DEFAULT 'Completed',
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    date TIMESTAMPTZ NOT NULL DEFAULT now(),
    description TEXT NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    notes TEXT,
    category_id UUID,
    to_account_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    user_id UUID NOT NULL,
    from_account_id UUID,
    receipt TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    account_id UUID NOT NULL,
    deleted_at TIMESTAMPTZ
);

CREATE TABLE user (
    is_active BOOLEAN NOT NULL DEFAULT true,
    deleted_at TIMESTAMPTZ,
    first_name VARCHAR(100),
    password_hash VARCHAR(255) NOT NULL,
    last_name VARCHAR(100),
    role role NOT NULL DEFAULT 'member',
    profile_pic TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    email VARCHAR(255) NOT NULL
);

CREATE INDEX account_name_user_idx ON account (name, user_id);
CREATE INDEX account_user_idx ON account (user_id);
CREATE INDEX budget_category_idx ON budget (category_id);
CREATE INDEX budget_date_range_idx ON budget (start_date, end_date);
CREATE INDEX budget_period_idx ON budget (period);
CREATE INDEX budget_user_idx ON budget (user_id);
CREATE INDEX category_type_idx ON category (type);
CREATE INDEX category_user_idx ON category (user_id);
CREATE INDEX goal_status_idx ON goal (status);
CREATE INDEX goal_target_date_idx ON goal (target_date);
CREATE INDEX goal_user_idx ON goal (user_id);
CREATE INDEX recurring_transaction_account_idx ON recurringtransaction (account_id);
CREATE INDEX recurring_transaction_category_idx ON recurringtransaction (category_id);
CREATE INDEX recurring_transaction_frequency_idx ON recurringtransaction (frequency);
CREATE INDEX recurring_transaction_next_date_idx ON recurringtransaction (next_date);
CREATE INDEX recurring_transaction_user_idx ON recurringtransaction (user_id);
CREATE INDEX transaction_account_idx ON transaction (account_id);
CREATE INDEX transaction_category_idx ON transaction (category_id);
CREATE INDEX transaction_date_idx ON transaction (date);
CREATE INDEX transaction_type_idx ON transaction (type);
CREATE INDEX transaction_user_idx ON transaction (user_id);
CREATE UNIQUE INDEX user_email_unique ON user (email);
ALTER TABLE account ADD CONSTRAINT account_name_user_unique UNIQUE (name, user_id);
ALTER TABLE budget ADD CONSTRAINT budget_name_user_unique UNIQUE (name, user_id);
ALTER TABLE category ADD CONSTRAINT category_name_user_unique UNIQUE (name, user_id);
ALTER TABLE goal ADD CONSTRAINT goal_name_user_unique UNIQUE (name, user_id);
ALTER TABLE recurringtransaction ADD CONSTRAINT recurring_transaction_name_user_unique UNIQUE (name, user_id);
ALTER TABLE account ADD CONSTRAINT account_balance_positive CHECK (balance >= 0);
ALTER TABLE account ADD CONSTRAINT account_name_length CHECK (char_length(name) > 0);
ALTER TABLE budget ADD CONSTRAINT budget_amount_positive CHECK (amount > 0);
ALTER TABLE budget ADD CONSTRAINT budget_date_range CHECK (end_date IS NULL OR end_date > start_date);
ALTER TABLE budget ADD CONSTRAINT budget_name_length CHECK (char_length(name) > 0);
ALTER TABLE category ADD CONSTRAINT category_color_format CHECK (color ~ '^#[0-9A-Fa-f]{6}$' OR color IS NULL);
ALTER TABLE category ADD CONSTRAINT category_name_length CHECK (char_length(name) > 0);
ALTER TABLE goal ADD CONSTRAINT goal_color_format CHECK (color ~ '^#[0-9A-Fa-f]{6}$' OR color IS NULL);
ALTER TABLE goal ADD CONSTRAINT goal_current_amount_positive CHECK (current_amount >= 0);
ALTER TABLE goal ADD CONSTRAINT goal_current_not_exceed_target CHECK (current_amount <= target_amount);
ALTER TABLE goal ADD CONSTRAINT goal_name_length CHECK (char_length(name) > 0);
ALTER TABLE goal ADD CONSTRAINT goal_target_amount_positive CHECK (target_amount > 0);
ALTER TABLE recurringtransaction ADD CONSTRAINT recurring_transaction_amount_positive CHECK (amount > 0);
ALTER TABLE recurringtransaction ADD CONSTRAINT recurring_transaction_date_range CHECK (end_date IS NULL OR end_date > start_date);
ALTER TABLE recurringtransaction ADD CONSTRAINT recurring_transaction_description_length CHECK (char_length(description) > 0);
ALTER TABLE recurringtransaction ADD CONSTRAINT recurring_transaction_name_length CHECK (char_length(name) > 0);
ALTER TABLE recurringtransaction ADD CONSTRAINT recurring_transaction_next_date_valid CHECK (next_date >= start_date);
ALTER TABLE recurringtransaction ADD CONSTRAINT transfer_accounts_different CHECK ((type != 'Transfer') OR (from_account_id != to_account_id));
ALTER TABLE recurringtransaction ADD CONSTRAINT transfer_accounts_required CHECK ((type != 'Transfer') OR (from_account_id IS NOT NULL AND to_account_id IS NOT NULL));
ALTER TABLE transaction ADD CONSTRAINT transaction_amount_positive CHECK (amount > 0);
ALTER TABLE transaction ADD CONSTRAINT transaction_description_length CHECK (char_length(description) > 0);
ALTER TABLE transaction ADD CONSTRAINT transfer_accounts_different CHECK ((type != 'Transfer') OR (from_account_id != to_account_id));
ALTER TABLE transaction ADD CONSTRAINT transfer_accounts_required CHECK ((type != 'Transfer') OR (from_account_id IS NOT NULL AND to_account_id IS NOT NULL));
ALTER TABLE user ADD CONSTRAINT user_email_not_empty CHECK (email <> '');
ALTER TABLE user ADD CONSTRAINT user_name_length CHECK (char_length(first_name) > 0 OR first_name IS NULL);
ALTER TABLE account ADD CONSTRAINT transactions FOREIGN KEY (id) REFERENCES transaction.account_id
ALTER TABLE account ADD CONSTRAINT transfers_from FOREIGN KEY (id) REFERENCES transaction.from_account_id
ALTER TABLE account ADD CONSTRAINT transfers_to FOREIGN KEY (id) REFERENCES transaction.to_account_id
ALTER TABLE account ADD CONSTRAINT user FOREIGN KEY (user_id) REFERENCES user.id
ALTER TABLE budget ADD CONSTRAINT category FOREIGN KEY (category_id) REFERENCES category.id
ALTER TABLE budget ADD CONSTRAINT user FOREIGN KEY (user_id) REFERENCES user.id
ALTER TABLE category ADD CONSTRAINT budgets FOREIGN KEY (id) REFERENCES budget.category_id
ALTER TABLE category ADD CONSTRAINT transactions FOREIGN KEY (id) REFERENCES transaction.category_id
ALTER TABLE category ADD CONSTRAINT user FOREIGN KEY (user_id) REFERENCES user.id
ALTER TABLE goal ADD CONSTRAINT user FOREIGN KEY (user_id) REFERENCES user.id
ALTER TABLE recurringtransaction ADD CONSTRAINT account FOREIGN KEY (account_id) REFERENCES account.id
ALTER TABLE recurringtransaction ADD CONSTRAINT category FOREIGN KEY (category_id) REFERENCES category.id
ALTER TABLE recurringtransaction ADD CONSTRAINT from_account FOREIGN KEY (from_account_id) REFERENCES account.id
ALTER TABLE recurringtransaction ADD CONSTRAINT to_account FOREIGN KEY (to_account_id) REFERENCES account.id
ALTER TABLE recurringtransaction ADD CONSTRAINT user FOREIGN KEY (user_id) REFERENCES user.id
ALTER TABLE transaction ADD CONSTRAINT account FOREIGN KEY (account_id) REFERENCES account.id
ALTER TABLE transaction ADD CONSTRAINT category FOREIGN KEY (category_id) REFERENCES category.id
ALTER TABLE transaction ADD CONSTRAINT from_account FOREIGN KEY (from_account_id) REFERENCES account.id
ALTER TABLE transaction ADD CONSTRAINT to_account FOREIGN KEY (to_account_id) REFERENCES account.id
ALTER TABLE transaction ADD CONSTRAINT user FOREIGN KEY (user_id) REFERENCES user.id
ALTER TABLE user ADD CONSTRAINT accounts FOREIGN KEY (id) REFERENCES account.user_id
ALTER TABLE user ADD CONSTRAINT budgets FOREIGN KEY (id) REFERENCES budget.user_id
ALTER TABLE user ADD CONSTRAINT goals FOREIGN KEY (id) REFERENCES goal.user_id
ALTER TABLE user ADD CONSTRAINT transactions FOREIGN KEY (id) REFERENCES transaction.user_id
COMMIT;
