use chrono::Utc;
use spl_domain::entities::user::Role;

#[test]
fn test_role_hierarchy_comparison() {
    let admin_role = Role {
        id: 1,
        name: "admin".to_string(),
        level: 100,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let supervisor_role = Role {
        id: 2,
        name: "supervisor".to_string(),
        level: 50,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let user_role = Role {
        id: 3,
        name: "user".to_string(),
        level: 10,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Admin > Supervisor
    assert!(admin_role > supervisor_role);
    // Supervisor > User
    assert!(supervisor_role > user_role);
    // Admin > User
    assert!(admin_role > user_role);
}
