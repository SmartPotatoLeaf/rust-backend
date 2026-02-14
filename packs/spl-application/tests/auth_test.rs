use async_trait::async_trait;
use mockall::mock;
use mockall::predicate::*;
use spl_application::dtos::user::LoginDto;
use spl_application::services::auth::AuthService;
use spl_domain::entities::user::{Role, User};
use spl_domain::ports::auth::{PasswordEncoder, TokenGenerator};
use spl_domain::ports::repositories::crud::CrudRepository;
use spl_domain::ports::repositories::user::{RoleRepository, UserRepository};
use spl_shared::error::Result;
use std::sync::Arc;
use uuid::Uuid;

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
    pub TokenGenerator {}
    impl TokenGenerator for TokenGenerator {
        fn generate(&self, sub: &str, claims: serde_json::Value) -> Result<String>;
        fn validate(&self, token: &str) -> Result<serde_json::Value>;
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

#[tokio::test]
async fn test_login_success() {
    let mut mock_repo = MockUserRepository::new();
    let mut mock_role_repo = MockRoleRepository::new();
    let mut mock_encoder = MockPasswordEncoder::new();
    let mut mock_token = MockTokenGenerator::new();

    let user_id = Uuid::new_v4();
    let user = User {
        id: user_id,
        username: "testuser".to_string(),
        email: Some("test@example.com".to_string()),
        password_hash: "hashed_secret".to_string(),
        name: None,
        surname: None,
        role: Role {
            id: ROLE_USER_ID,
            name: "User".to_string(),
            level: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        company: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let role = Role {
        id: ROLE_USER_ID,
        name: "User".to_string(),
        level: 1,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    mock_repo
        .expect_get_by_username_or_email_and_company()
        .with(eq(Some("testuser".to_string())), eq(None), eq(None))
        .times(1)
        .returning(move |_, _, _| Ok(Some(user.clone())));

    mock_encoder
        .expect_verify()
        .with(eq("secret"), eq("hashed_secret"))
        .times(1)
        .returning(|_, _| Ok(true));

    mock_token
        .expect_generate()
        .with(
            always(), // subject
            function(|claims: &serde_json::Value| claims["role"] == "User"),
        )
        .times(1)
        .returning(|_, _| Ok("jwt_token".to_string()));

    let service = AuthService::new(
        Arc::new(mock_repo),
        Arc::new(mock_encoder),
        Arc::new(mock_token),
    );

    let login_dto = LoginDto {
        username: Some("testuser".to_string()),
        email: None,
        password: "secret".to_string(),
        company_id: None,
    };

    let result = service.login(login_dto).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "jwt_token");
}
