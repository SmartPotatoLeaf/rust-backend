use async_trait::async_trait;
use mockall::mock;
use mockall::predicate::*;
use spl_application::dtos::user::CreateUserDto;
use spl_application::services::access_control::AccessControlService;
use spl_application::services::user::UserService;
use spl_domain::entities::company::Company;
use spl_domain::entities::user::{Role, User};
use spl_domain::ports::auth::PasswordEncoder;
use spl_domain::ports::repositories::company::CompanyRepository;
use spl_domain::ports::repositories::crud::CrudRepository;
use spl_domain::ports::repositories::user::{RoleRepository, UserRepository};
use spl_shared::error::Result;
use std::sync::Arc;
use uuid::Uuid;

const ROLE_ADMIN_ID: i32 = 1;
const ROLE_SUPERVISOR_ID: i32 = 2;
const ROLE_USER_ID: i32 = 3;

// Mock definitions
mock! {
    pub UserRepository {}
    #[async_trait]
    impl CrudRepository<User, Uuid> for UserRepository {
        async fn get_by_id(&self, id: Uuid) -> Result<Option<User>>;
        async fn create(&self, entity: User) -> Result<User>;
        async fn update(&self, entity: User) -> Result<User>;
        async fn delete(&self, id: Uuid) -> Result<User>;
    }
    #[async_trait]
    impl UserRepository for UserRepository {
        async fn get_by_ids(&self, ids: Vec<Uuid>) -> Result<Vec<User>>;
        async fn get_by_username_and_company(&self, username: &str, company_id: Option<Uuid>) -> Result<Option<User>>;
        async fn get_by_username_or_email_and_company(&self, username: Option<String>, email: Option<String>, company_id: Option<Uuid>) -> Result<Option<User>>;
        async fn get_by_company_id(&self, company_id: Uuid) -> Result<Vec<User>> ;
    }
}

mock! {
    pub PasswordEncoder {}
    impl PasswordEncoder for PasswordEncoder {
        fn hash(&self, password: &str) -> Result<String>;
        fn verify(&self, password: &str, hash: &str) -> Result<bool>;
    }
}

mock! {
    pub RoleRepository {}
    #[async_trait]
    impl CrudRepository<Role, i32> for RoleRepository {
        async fn get_by_id(&self, id: i32) -> Result<Option<Role>>;
        async fn create(&self, entity: Role) -> Result<Role>;
        async fn update(&self, entity: Role) -> Result<Role>;
        async fn delete(&self, id: i32) -> Result<Role>;
    }
    #[async_trait]
    impl RoleRepository for RoleRepository {
        async fn get_by_name(&self, name: &str) -> Result<Option<Role>>;
        async fn get_all(&self) -> Result<Vec<Role>>;
    }
}

mock! {
    pub CompanyRepository {}
    #[async_trait]
    impl CrudRepository<Company, Uuid> for CompanyRepository {
        async fn get_by_id(&self, id: Uuid) -> Result<Option<Company>>;
        async fn create(&self, entity: Company) -> Result<Company>;
        async fn update(&self, entity: Company) -> Result<Company>;
        async fn delete(&self, id: Uuid) -> Result<Company>;
    }
    #[async_trait]
    impl CompanyRepository for CompanyRepository {
        async fn get_all(&self) -> Result<Vec<Company>>;
    }
}

#[tokio::test]
async fn test_create_user_admin_creates_admin_success() {
    let mut mock_repo = MockUserRepository::new();
    let mut mock_role_repo = MockRoleRepository::new();
    let mut mock_company_repo = MockCompanyRepository::new();
    let mut mock_encoder = MockPasswordEncoder::new();

    let company_id = Uuid::new_v4();

    let admin_user = User {
        id: Uuid::new_v4(),
        username: "admin".to_string(),
        email: Some("admin@example.com".to_string()),
        password_hash: "hash".to_string(),
        name: None,
        surname: None,
        role: Role {
            id: ROLE_ADMIN_ID,
            name: "admin".to_string(),
            level: 100,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        company: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let new_admin = User {
        id: Uuid::new_v4(),
        username: "newadmin".to_string(),
        email: Some("new@example.com".to_string()),
        password_hash: "hashed".to_string(),
        name: None,
        surname: None,
        role: Role {
            id: ROLE_ADMIN_ID,
            name: "admin".to_string(),
            level: 100,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        company: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // ...

    // Supervisor Test Setup

    let supervisor_company = Company {
        id: company_id,
        name: "Sup Corp".to_string(),
        description: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let supervisor = User {
        id: Uuid::new_v4(),
        username: "supervisor".to_string(),
        email: Some("sup@example.com".to_string()),
        password_hash: "hash".to_string(),
        name: None,
        surname: None,
        role: Role {
            id: ROLE_SUPERVISOR_ID,
            name: "supervisor".to_string(),
            level: 50,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        company: Some(supervisor_company.clone()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let new_user = User {
        id: Uuid::new_v4(),
        username: "user".to_string(),
        email: Some("user@example.com".to_string()),
        password_hash: "hashed".to_string(),
        name: None,
        surname: None,
        role: Role {
            id: ROLE_USER_ID,
            name: "user".to_string(),
            level: 10,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        company: Some(supervisor_company.clone()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Mock Role Repo Expectations
    mock_role_repo
        .expect_get_by_id()
        .with(eq(ROLE_ADMIN_ID))
        .returning(|_| {
            Ok(Some(Role {
                id: ROLE_ADMIN_ID,
                name: "admin".to_string(),
                level: 100,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        });

    mock_role_repo
        .expect_get_by_name()
        .with(eq("admin"))
        .returning(|_| {
            Ok(Some(Role {
                id: ROLE_ADMIN_ID,
                name: "admin".to_string(),
                level: 100,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        });

    mock_repo
        .expect_get_by_username_and_company()
        .with(eq("newadmin"), eq(None))
        .returning(|_, _| Ok(None)); // Not found

    mock_repo
        .expect_create()
        .returning(move |_| Ok(new_admin.clone()));

    mock_encoder
        .expect_hash()
        .returning(|_| Ok("hashed".to_string()));

    let mock_repo = Arc::new(mock_repo);
    let mock_company_repo = Arc::new(mock_company_repo);
    let access_control = Arc::new(AccessControlService::new(
        mock_company_repo.clone(),
        mock_repo.clone(),
    ));

    let service = UserService::new(
        mock_repo,
        Arc::new(mock_role_repo),
        mock_company_repo,
        Arc::new(mock_encoder),
        access_control,
    );

    let dto = CreateUserDto {
        username: "newadmin".to_string(),
        email: Some("new@example.com".to_string()),
        password: "pass".to_string(),
        name: None,
        surname: None,
        company_id: None,
        role: Some("admin".to_string()),
    };

    let result = service.create_user(&admin_user, dto).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_create_user_supervisor_creates_user_success() {
    let mut mock_repo = MockUserRepository::new();
    let mut mock_role_repo = MockRoleRepository::new();
    let mut mock_company_repo = MockCompanyRepository::new();
    let mut mock_encoder = MockPasswordEncoder::new();

    let company_id = Uuid::new_v4();
    let supervisor_company = Company {
        id: company_id,
        name: "Sup Corp".to_string(),
        description: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let supervisor = User {
        id: Uuid::new_v4(),
        username: "supervisor".to_string(),
        email: Some("sup@example.com".to_string()),
        password_hash: "hash".to_string(),
        name: None,
        surname: None,
        role: Role {
            id: ROLE_SUPERVISOR_ID,
            name: "supervisor".to_string(),
            level: 50,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        company: Some(supervisor_company.clone()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let new_user = User {
        id: Uuid::new_v4(),
        username: "user".to_string(),
        email: Some("user@example.com".to_string()),
        password_hash: "hashed".to_string(),
        name: None,
        surname: None,
        role: Role {
            id: ROLE_USER_ID,
            name: "user".to_string(),
            level: 10,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        company: Some(supervisor_company.clone()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Mock Role Repo Expectations
    // 1. Fetch supervisor role
    // NOTE: UserService reads role FROM USER object directly now. So this might not be called if we don't reload creator.
    // However, create_user DOES NOT reload creator. It uses `creator.role`.

    // 2. Fetch target role (User)
    mock_role_repo
        .expect_get_by_name()
        .with(eq("user"))
        .returning(|_| {
            Ok(Some(Role {
                id: ROLE_USER_ID,
                name: "user".to_string(),
                level: 10,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        });

    mock_role_repo
        .expect_get_by_id()
        .with(eq(ROLE_USER_ID))
        .returning(|_| {
            Ok(Some(Role {
                id: ROLE_USER_ID,
                name: "user".to_string(),
                level: 10,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        });

    mock_company_repo
        .expect_get_by_id()
        .with(eq(company_id))
        .returning(move |_| {
            Ok(Some(Company {
                id: company_id,
                name: "Sup Corp".to_string(),
                description: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        });

    mock_repo
        .expect_get_by_username_and_company()
        .with(eq("user"), eq(Some(company_id))) // Should check against supervisor's company
        .returning(|_, _| Ok(None));

    mock_repo
        .expect_create()
        .returning(move |_| Ok(new_user.clone()));

    mock_encoder
        .expect_hash()
        .returning(|_| Ok("hashed".to_string()));

    let mock_repo = Arc::new(mock_repo);
    let mock_company_repo = Arc::new(mock_company_repo);
    let access_control = Arc::new(AccessControlService::new(
        mock_company_repo.clone(),
        mock_repo.clone(),
    ));

    let service = UserService::new(
        mock_repo,
        Arc::new(mock_role_repo),
        mock_company_repo,
        Arc::new(mock_encoder),
        access_control,
    );

    let dto = CreateUserDto {
        username: "user".to_string(),
        email: Some("user@example.com".to_string()),
        password: "pass".to_string(),
        name: None,
        surname: None,
        company_id: None,
        role: Some("user".to_string()),
    };

    let result = service.create_user(&supervisor, dto).await;

    assert!(result.is_ok());
    let created = result.unwrap();
    // Verify company object presence
    assert!(created.company.is_some());
    assert_eq!(created.company.unwrap().id, company_id);
}
