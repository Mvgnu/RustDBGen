use crate::generated::{Route, Permissions};

pub mod routes {
    use super::*;

    pub const ACCOUNT: Route = Route { methods: &["GET", "POST", "PUT", "DELETE"], path: "/api/accounts", auth_required: true, permissions: Permissions { read: &["admin", "member"], update: &["admin", "member"], delete: &["admin", "member"] } };
    pub const BUDGET: Route = Route { methods: &["GET", "POST", "PUT", "DELETE"], path: "/api/budgets", auth_required: true, permissions: Permissions { read: &["admin", "member"], update: &["admin", "member"], delete: &["admin", "member"] } };
    pub const CATEGORY: Route = Route { methods: &["GET", "POST", "PUT", "DELETE"], path: "/api/categories", auth_required: true, permissions: Permissions { read: &["admin", "member", "public"], update: &["admin", "member"], delete: &["admin", "member"] } };
    pub const GOAL: Route = Route { methods: &["GET", "POST", "PUT", "DELETE"], path: "/api/goals", auth_required: true, permissions: Permissions { read: &["admin", "member"], update: &["admin", "member"], delete: &["admin", "member"] } };
    pub const RECURRINGTRANSACTION: Route = Route { methods: &["GET", "POST", "PUT", "DELETE"], path: "/api/recurring-transactions", auth_required: true, permissions: Permissions { read: &["admin", "member"], update: &["admin", "member"], delete: &["admin", "member"] } };
    pub const TRANSACTION: Route = Route { methods: &["GET", "POST", "PUT", "DELETE"], path: "/api/transactions", auth_required: true, permissions: Permissions { read: &["admin", "member"], update: &["admin", "member"], delete: &["admin", "member"] } };
    pub const USER: Route = Route { methods: &["GET", "POST", "PUT", "DELETE"], path: "/api/users", auth_required: true, permissions: Permissions { read: &["admin", "member"], update: &["admin", "member"], delete: &["admin"] } };
}

