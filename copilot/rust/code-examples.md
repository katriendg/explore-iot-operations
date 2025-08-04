# Comprehensive Code Examples and Best Practices

**Core Principles:**
- Provide complete, working examples for common patterns
- Show both basic and advanced usage scenarios
- Include error handling and edge cases
- Demonstrate performance considerations
- Follow established Rust conventions

## Complete HTTP Client Example

```rust
// Cargo.toml
// [dependencies]
// tokio = { version = "1.0", features = ["full"] }
// reqwest = { version = "0.11", features = ["json"] }
// serde = { version = "1.0", features = ["derive"] }
// anyhow = "1.0"

use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use anyhow::{Result, Context};

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    status: String,
    data: Option<serde_json::Value>,
    message: Option<String>,
}

#[derive(Debug, Serialize)]
struct ApiRequest {
    action: String,
    payload: serde_json::Value,
}

pub struct ApiClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl ApiClient {
    pub fn new(base_url: String, api_key: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(ApiClient {
            client,
            base_url,
            api_key,
        })
    }

    pub async fn get<T>(&self, endpoint: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{}/{}", self.base_url, endpoint);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .send()
            .await
            .context("Failed to send GET request")?;

        self.handle_response(response).await
    }

    pub async fn post<T, U>(&self, endpoint: &str, data: &T) -> Result<U>
    where
        T: Serialize,
        U: for<'de> Deserialize<'de>,
    {
        let url = format!("{}/{}", self.base_url, endpoint);
        
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(data)
            .send()
            .await
            .context("Failed to send POST request")?;

        self.handle_response(response).await
    }

    async fn handle_response<T>(&self, response: Response) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let status = response.status();
        let text = response
            .text()
            .await
            .context("Failed to read response body")?;

        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "API request failed with status {}: {}",
                status,
                text
            ));
        }

        serde_json::from_str(&text)
            .context("Failed to parse JSON response")
    }
}

// Usage example
#[tokio::main]
async fn main() -> Result<()> {
    let client = ApiClient::new(
        "https://api.example.com".to_string(),
        "your-api-key".to_string(),
    )?;

    // GET request
    let user_data: serde_json::Value = client
        .get("users/123")
        .await
        .context("Failed to fetch user data")?;

    println!("User data: {:#?}", user_data);

    // POST request
    let request = ApiRequest {
        action: "create_user".to_string(),
        payload: serde_json::json!({
            "name": "John Doe",
            "email": "john@example.com"
        }),
    };

    let response: ApiResponse = client
        .post("users", &request)
        .await
        .context("Failed to create user")?;

    println!("Create user response: {:#?}", response);

    Ok(())
}
```

## Configuration Management Pattern

```rust
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub max_connections: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: u32,
    pub request_timeout_ms: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub log_level: String,
    pub environment: Environment,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                workers: 4,
                request_timeout_ms: 30000,
            },
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                username: "user".to_string(),
                password: "password".to_string(),
                database: "myapp".to_string(),
                max_connections: 10,
            },
            log_level: "info".to_string(),
            environment: Environment::Development,
        }
    }
}

impl AppConfig {
    /// Load configuration from multiple sources with precedence:
    /// 1. Environment variables (highest)
    /// 2. Config file
    /// 3. Default values (lowest)
    pub fn load() -> Result<Self> {
        let mut config = Self::default();

        // Load from config file if it exists
        if let Ok(config_path) = env::var("CONFIG_FILE") {
            if Path::new(&config_path).exists() {
                config = Self::from_file(&config_path)?;
            }
        } else if Path::new("config.toml").exists() {
            config = Self::from_file("config.toml")?;
        }

        // Override with environment variables
        config.apply_env_overrides()?;

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)
            .context(format!("Failed to read config file: {}", path))?;

        toml::from_str(&content)
            .context("Failed to parse config file")
    }

    fn apply_env_overrides(&mut self) -> Result<()> {
        // Server configuration
        if let Ok(host) = env::var("SERVER_HOST") {
            self.server.host = host;
        }
        if let Ok(port) = env::var("SERVER_PORT") {
            self.server.port = port.parse()
                .context("Invalid SERVER_PORT")?;
        }

        // Database configuration
        if let Ok(db_host) = env::var("DATABASE_HOST") {
            self.database.host = db_host;
        }
        if let Ok(db_port) = env::var("DATABASE_PORT") {
            self.database.port = db_port.parse()
                .context("Invalid DATABASE_PORT")?;
        }
        if let Ok(db_user) = env::var("DATABASE_USERNAME") {
            self.database.username = db_user;
        }
        if let Ok(db_pass) = env::var("DATABASE_PASSWORD") {
            self.database.password = db_pass;
        }

        // Environment
        if let Ok(env_str) = env::var("ENVIRONMENT") {
            self.environment = serde_json::from_str(&format!("\"{}\"", env_str.to_lowercase()))
                .context("Invalid ENVIRONMENT value")?;
        }

        Ok(())
    }

    fn validate(&self) -> Result<()> {
        if self.server.port == 0 {
            return Err(anyhow::anyhow!("Server port cannot be 0"));
        }
        if self.database.port == 0 {
            return Err(anyhow::anyhow!("Database port cannot be 0"));
        }
        if self.database.username.is_empty() {
            return Err(anyhow::anyhow!("Database username cannot be empty"));
        }
        if self.server.workers == 0 {
            return Err(anyhow::anyhow!("Server workers must be > 0"));
        }

        Ok(())
    }

    pub fn is_production(&self) -> bool {
        matches!(self.environment, Environment::Production)
    }
}
```

## Generic Data Processing Pipeline

```rust
use std::marker::PhantomData;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::mpsc;
use anyhow::Result;

// Trait for processing steps
#[async_trait]
pub trait Processor<T, U>: Send + Sync {
    async fn process(&self, input: T) -> Result<U>;
}

// Generic pipeline that chains processors
pub struct Pipeline<T> {
    _phantom: PhantomData<T>,
}

impl<T> Pipeline<T>
where
    T: Send + 'static,
{
    pub fn new() -> Self {
        Pipeline {
            _phantom: PhantomData,
        }
    }

    pub async fn process_stream<U, P>(
        &self,
        mut input_rx: mpsc::Receiver<T>,
        processor: Arc<P>,
        output_tx: mpsc::Sender<U>,
    ) -> Result<()>
    where
        U: Send + 'static,
        P: Processor<T, U> + 'static,
    {
        while let Some(item) = input_rx.recv().await {
            match processor.process(item).await {
                Ok(result) => {
                    if output_tx.send(result).await.is_err() {
                        break; // Receiver dropped
                    }
                }
                Err(e) => {
                    eprintln!("Processing error: {}", e);
                    // Continue processing other items
                }
            }
        }
        Ok(())
    }
}

// Example processors
pub struct StringToUppercase;

#[async_trait]
impl Processor<String, String> for StringToUppercase {
    async fn process(&self, input: String) -> Result<String> {
        Ok(input.to_uppercase())
    }
}

pub struct NumberDoubler;

#[async_trait]
impl Processor<i32, i32> for NumberDoubler {
    async fn process(&self, input: i32) -> Result<i32> {
        Ok(input * 2)
    }
}

// Usage example
async fn pipeline_example() -> Result<()> {
    let (input_tx, input_rx) = mpsc::channel::<String>(100);
    let (output_tx, mut output_rx) = mpsc::channel::<String>(100);

    let pipeline = Pipeline::new();
    let processor = Arc::new(StringToUppercase);

    // Start processing pipeline
    let processing_task = tokio::spawn(async move {
        pipeline.process_stream(input_rx, processor, output_tx).await
    });

    // Send data
    input_tx.send("hello".to_string()).await?;
    input_tx.send("world".to_string()).await?;
    drop(input_tx); // Close input

    // Receive results
    while let Some(result) = output_rx.recv().await {
        println!("Processed: {}", result);
    }

    processing_task.await??;
    Ok(())
}
```

## Error Handling with Custom Error Types

```rust
use thiserror::Error;
use std::fmt;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {source}")]
    Database {
        #[from]
        source: sqlx::Error,
    },

    #[error("Network error: {message}")]
    Network { message: String },

    #[error("Validation error in field '{field}': {message}")]
    Validation { field: String, message: String },

    #[error("Not found: {resource}")]
    NotFound { resource: String },

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Permission denied: {action}")]
    PermissionDenied { action: String },

    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    pub fn validation(field: &str, message: &str) -> Self {
        AppError::Validation {
            field: field.to_string(),
            message: message.to_string(),
        }
    }

    pub fn not_found(resource: &str) -> Self {
        AppError::NotFound {
            resource: resource.to_string(),
        }
    }

    pub fn network(message: &str) -> Self {
        AppError::Network {
            message: message.to_string(),
        }
    }

    pub fn permission_denied(action: &str) -> Self {
        AppError::PermissionDenied {
            action: action.to_string(),
        }
    }

    pub fn status_code(&self) -> u16 {
        match self {
            AppError::Database { .. } => 500,
            AppError::Network { .. } => 502,
            AppError::Validation { .. } => 400,
            AppError::NotFound { .. } => 404,
            AppError::AuthenticationFailed => 401,
            AppError::PermissionDenied { .. } => 403,
            AppError::Internal(_) => 500,
        }
    }

    pub fn user_message(&self) -> String {
        match self {
            AppError::Database { .. } => "A database error occurred".to_string(),
            AppError::Network { .. } => "A network error occurred".to_string(),
            AppError::Validation { field, message } => {
                format!("Validation error in {}: {}", field, message)
            }
            AppError::NotFound { resource } => format!("{} not found", resource),
            AppError::AuthenticationFailed => "Authentication required".to_string(),
            AppError::PermissionDenied { action } => {
                format!("Permission denied for action: {}", action)
            }
            AppError::Internal(_) => "An internal error occurred".to_string(),
        }
    }
}

// Usage in application functions
type AppResult<T> = Result<T, AppError>;

pub struct UserService {
    // ... fields
}

impl UserService {
    pub async fn get_user(&self, id: u64) -> AppResult<User> {
        // Validate input
        if id == 0 {
            return Err(AppError::validation("id", "ID must be greater than 0"));
        }

        // Fetch from database
        let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
            .fetch_optional(&self.db)
            .await?
            .ok_or_else(|| AppError::not_found("User"))?;

        Ok(user)
    }

    pub async fn create_user(&self, data: CreateUserRequest) -> AppResult<User> {
        // Validate request
        self.validate_create_request(&data)?;

        // Check if user already exists
        if self.user_exists(&data.email).await? {
            return Err(AppError::validation("email", "Email already exists"));
        }

        // Create user
        let user = sqlx::query_as!(
            User,
            "INSERT INTO users (email, name) VALUES ($1, $2) RETURNING *",
            data.email,
            data.name
        )
        .fetch_one(&self.db)
        .await?;

        Ok(user)
    }

    fn validate_create_request(&self, data: &CreateUserRequest) -> AppResult<()> {
        if data.email.is_empty() {
            return Err(AppError::validation("email", "Email is required"));
        }
        if !data.email.contains('@') {
            return Err(AppError::validation("email", "Invalid email format"));
        }
        if data.name.is_empty() {
            return Err(AppError::validation("name", "Name is required"));
        }
        Ok(())
    }

    async fn user_exists(&self, email: &str) -> AppResult<bool> {
        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM users WHERE email = $1",
            email
        )
        .fetch_one(&self.db)
        .await?;

        Ok(count > 0)
    }
}
```

## Resource Management with RAII

```rust
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;

pub struct Resource {
    id: String,
    data: Vec<u8>,
}

impl Resource {
    fn new(id: String, size: usize) -> Self {
        Resource {
            id,
            data: vec![0; size],
        }
    }

    fn cleanup(&mut self) {
        println!("Cleaning up resource: {}", self.id);
        self.data.clear();
    }
}

impl Drop for Resource {
    fn drop(&mut self) {
        self.cleanup();
    }
}

// Resource pool for managing multiple resources
pub struct ResourcePool {
    resources: Mutex<Vec<Resource>>,
    max_size: usize,
}

impl ResourcePool {
    pub fn new(max_size: usize) -> Arc<Self> {
        Arc::new(ResourcePool {
            resources: Mutex::new(Vec::new()),
            max_size,
        })
    }

    pub async fn acquire(&self, id: String) -> Result<ResourceGuard> {
        let mut resources = self.resources.lock().await;
        
        if resources.len() < self.max_size {
            let resource = Resource::new(id.clone(), 1024);
            resources.push(resource);
            
            let index = resources.len() - 1;
            Ok(ResourceGuard {
                pool: self as *const ResourcePool,
                index,
                id,
            })
        } else {
            Err(anyhow::anyhow!("Resource pool exhausted"))
        }
    }

    async fn release(&self, index: usize) {
        let mut resources = self.resources.lock().await;
        if index < resources.len() {
            resources.remove(index);
        }
    }
}

// RAII guard for automatic resource cleanup
pub struct ResourceGuard {
    pool: *const ResourcePool,
    index: usize,
    id: String,
}

impl ResourceGuard {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub async fn process_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Simulate processing
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        let mut result = data.to_vec();
        result.reverse();
        Ok(result)
    }
}

impl Drop for ResourceGuard {
    fn drop(&mut self) {
        // Note: In real code, you'd want to use a channel or other async mechanism
        // for cleanup since Drop is not async
        println!("Releasing resource: {}", self.id);
    }
}

// Usage example
async fn resource_example() -> Result<()> {
    let pool = ResourcePool::new(5);
    
    {
        let resource = pool.acquire("test-resource-1".to_string()).await?;
        let result = resource.process_data(b"hello world").await?;
        println!("Processed: {:?}", String::from_utf8_lossy(&result));
        
        // Resource automatically released when guard goes out of scope
    }
    
    Ok(())
}
```

## Architectural Guidelines

**Core Principles:**
- Design for modularity and composability
- Separate concerns with clear boundaries
- Use appropriate abstraction levels
- Follow dependency inversion principle
- Design APIs that are hard to misuse

### Module Organization Patterns

```rust
// src/lib.rs - Library root
//! # MyApp Library
//! 
//! This library provides core functionality for the MyApp application.

pub mod config;
pub mod database;
pub mod services;
pub mod models;
pub mod errors;
pub mod utils;

// Re-export commonly used types
pub use errors::{AppError, AppResult};
pub use config::AppConfig;

// Prelude module for common imports
pub mod prelude {
    pub use crate::{AppError, AppResult, AppConfig};
    pub use crate::services::UserService;
    pub use crate::models::{User, CreateUserRequest};
}

// src/models/mod.rs - Data models
pub mod user;
pub mod organization;
pub mod common;

pub use user::{User, CreateUserRequest, UpdateUserRequest};
pub use organization::{Organization, CreateOrgRequest};
pub use common::{Id, Timestamp, Pagination};

// src/services/mod.rs - Business logic
mod user_service;
mod auth_service;
mod notification_service;

pub use user_service::UserService;
pub use auth_service::AuthService;
pub use notification_service::NotificationService;

// Create a service container
pub struct Services {
    pub user: UserService,
    pub auth: AuthService,
    pub notification: NotificationService,
}

impl Services {
    pub async fn new(config: &AppConfig, db: sqlx::Pool<sqlx::Postgres>) -> AppResult<Self> {
        Ok(Services {
            user: UserService::new(db.clone()).await?,
            auth: AuthService::new(config.auth.clone(), db.clone()).await?,
            notification: NotificationService::new(config.notification.clone()).await?,
        })
    }
}
```

### Layered Architecture Pattern

```rust
// Domain layer - Core business logic
pub mod domain {
    use async_trait::async_trait;
    use crate::errors::AppResult;

    // Domain entities
    #[derive(Debug, Clone)]
    pub struct User {
        pub id: UserId,
        pub email: String,
        pub name: String,
        pub created_at: chrono::DateTime<chrono::Utc>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct UserId(pub u64);

    // Domain services (business logic)
    pub struct UserDomainService {
        repository: Box<dyn UserRepository>,
        email_service: Box<dyn EmailService>,
    }

    impl UserDomainService {
        pub fn new(
            repository: Box<dyn UserRepository>,
            email_service: Box<dyn EmailService>,
        ) -> Self {
            UserDomainService {
                repository,
                email_service,
            }
        }

        pub async fn create_user(&self, email: String, name: String) -> AppResult<User> {
            // Business logic validation
            if !self.is_valid_email(&email) {
                return Err(AppError::validation("email", "Invalid email format"));
            }

            // Check business rules
            if self.repository.exists_by_email(&email).await? {
                return Err(AppError::validation("email", "Email already exists"));
            }

            // Create user
            let user = User {
                id: UserId(0), // Will be set by repository
                email: email.clone(),
                name,
                created_at: chrono::Utc::now(),
            };

            let saved_user = self.repository.save(user).await?;

            // Send welcome email (side effect)
            self.email_service.send_welcome_email(&saved_user).await?;

            Ok(saved_user)
        }

        fn is_valid_email(&self, email: &str) -> bool {
            email.contains('@') && email.len() > 3
        }
    }

    // Repository traits (domain interfaces)
    #[async_trait]
    pub trait UserRepository: Send + Sync {
        async fn save(&self, user: User) -> AppResult<User>;
        async fn find_by_id(&self, id: UserId) -> AppResult<Option<User>>;
        async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;
        async fn exists_by_email(&self, email: &str) -> AppResult<bool>;
        async fn delete(&self, id: UserId) -> AppResult<()>;
    }

    #[async_trait]
    pub trait EmailService: Send + Sync {
        async fn send_welcome_email(&self, user: &User) -> AppResult<()>;
        async fn send_password_reset(&self, user: &User, token: &str) -> AppResult<()>;
    }
}

// Infrastructure layer - External dependencies
pub mod infrastructure {
    use async_trait::async_trait;
    use sqlx::{PgPool, FromRow};
    use crate::domain::{User, UserId, UserRepository, EmailService};
    use crate::errors::AppResult;

    // Database implementation
    pub struct PostgresUserRepository {
        pool: PgPool,
    }

    impl PostgresUserRepository {
        pub fn new(pool: PgPool) -> Self {
            PostgresUserRepository { pool }
        }
    }

    #[derive(FromRow)]
    struct UserRow {
        id: i64,
        email: String,
        name: String,
        created_at: chrono::DateTime<chrono::Utc>,
    }

    impl From<UserRow> for User {
        fn from(row: UserRow) -> Self {
            User {
                id: UserId(row.id as u64),
                email: row.email,
                name: row.name,
                created_at: row.created_at,
            }
        }
    }

    #[async_trait]
    impl UserRepository for PostgresUserRepository {
        async fn save(&self, mut user: User) -> AppResult<User> {
            let row = sqlx::query_as!(
                UserRow,
                "INSERT INTO users (email, name, created_at) VALUES ($1, $2, $3) RETURNING *",
                user.email,
                user.name,
                user.created_at
            )
            .fetch_one(&self.pool)
            .await?;

            Ok(row.into())
        }

        async fn find_by_id(&self, id: UserId) -> AppResult<Option<User>> {
            let row = sqlx::query_as!(
                UserRow,
                "SELECT * FROM users WHERE id = $1",
                id.0 as i64
            )
            .fetch_optional(&self.pool)
            .await?;

            Ok(row.map(Into::into))
        }

        async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
            let row = sqlx::query_as!(
                UserRow,
                "SELECT * FROM users WHERE email = $1",
                email
            )
            .fetch_optional(&self.pool)
            .await?;

            Ok(row.map(Into::into))
        }

        async fn exists_by_email(&self, email: &str) -> AppResult<bool> {
            let count: i64 = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM users WHERE email = $1",
                email
            )
            .fetch_one(&self.pool)
            .await?;

            Ok(count > 0)
        }

        async fn delete(&self, id: UserId) -> AppResult<()> {
            sqlx::query!("DELETE FROM users WHERE id = $1", id.0 as i64)
                .execute(&self.pool)
                .await?;

            Ok(())
        }
    }

    // Email service implementation
    pub struct SmtpEmailService {
        smtp_host: String,
        smtp_port: u16,
        username: String,
        password: String,
    }

    impl SmtpEmailService {
        pub fn new(smtp_host: String, smtp_port: u16, username: String, password: String) -> Self {
            SmtpEmailService {
                smtp_host,
                smtp_port,
                username,
                password,
            }
        }
    }

    #[async_trait]
    impl EmailService for SmtpEmailService {
        async fn send_welcome_email(&self, user: &User) -> AppResult<()> {
            // Implementation would use an SMTP library
            println!("Sending welcome email to: {}", user.email);
            Ok(())
        }

        async fn send_password_reset(&self, user: &User, token: &str) -> AppResult<()> {
            println!("Sending password reset to: {} with token: {}", user.email, token);
            Ok(())
        }
    }
}

// Application layer - Use cases and coordination
pub mod application {
    use crate::domain::{UserDomainService, User, UserId};
    use crate::errors::AppResult;

    pub struct UserApplicationService {
        domain_service: UserDomainService,
    }

    impl UserApplicationService {
        pub fn new(domain_service: UserDomainService) -> Self {
            UserApplicationService { domain_service }
        }

        pub async fn register_user(&self, email: String, name: String) -> AppResult<UserId> {
            let user = self.domain_service.create_user(email, name).await?;
            Ok(user.id)
        }

        pub async fn get_user(&self, id: UserId) -> AppResult<Option<User>> {
            self.domain_service.repository.find_by_id(id).await
        }
    }
}
```

### API Design Patterns

```rust
// Builder pattern for complex configuration
pub struct DatabaseConnectionBuilder {
    host: Option<String>,
    port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
    database: Option<String>,
    max_connections: Option<u32>,
    timeout: Option<std::time::Duration>,
}

impl DatabaseConnectionBuilder {
    pub fn new() -> Self {
        DatabaseConnectionBuilder {
            host: None,
            port: None,
            username: None,
            password: None,
            database: None,
            max_connections: None,
            timeout: None,
        }
    }

    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn credentials(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self
    }

    pub fn database(mut self, database: impl Into<String>) -> Self {
        self.database = Some(database.into());
        self
    }

    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = Some(max);
        self
    }

    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub async fn connect(self) -> AppResult<DatabaseConnection> {
        let config = DatabaseConfig {
            host: self.host.unwrap_or_else(|| "localhost".to_string()),
            port: self.port.unwrap_or(5432),
            username: self.username.ok_or_else(|| {
                AppError::validation("username", "Username is required")
            })?,
            password: self.password.ok_or_else(|| {
                AppError::validation("password", "Password is required")
            })?,
            database: self.database.ok_or_else(|| {
                AppError::validation("database", "Database name is required")
            })?,
            max_connections: self.max_connections.unwrap_or(10),
            timeout: self.timeout.unwrap_or(std::time::Duration::from_secs(30)),
        };

        DatabaseConnection::new(config).await
    }
}

// Usage:
// let db = DatabaseConnectionBuilder::new()
//     .host("localhost")
//     .port(5432)
//     .credentials("user", "password")
//     .database("myapp")
//     .max_connections(20)
//     .connect()
//     .await?;

// Type-safe query builder
pub struct QueryBuilder<T> {
    table: String,
    conditions: Vec<String>,
    order_by: Vec<String>,
    limit: Option<usize>,
    offset: Option<usize>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> QueryBuilder<T> {
    pub fn from_table(table: &str) -> Self {
        QueryBuilder {
            table: table.to_string(),
            conditions: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn where_eq(mut self, column: &str, value: &str) -> Self {
        self.conditions.push(format!("{} = '{}'", column, value));
        self
    }

    pub fn order_by(mut self, column: &str, direction: OrderDirection) -> Self {
        let dir = match direction {
            OrderDirection::Asc => "ASC",
            OrderDirection::Desc => "DESC",
        };
        self.order_by.push(format!("{} {}", column, dir));
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn build(self) -> String {
        let mut query = format!("SELECT * FROM {}", self.table);

        if !self.conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&self.conditions.join(" AND "));
        }

        if !self.order_by.is_empty() {
            query.push_str(" ORDER BY ");
            query.push_str(&self.order_by.join(", "));
        }

        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        query
    }
}

pub enum OrderDirection {
    Asc,
    Desc,
}
```

### Dependency Injection Pattern

```rust
use std::sync::Arc;

// Container for dependency injection
pub struct ServiceContainer {
    user_service: Arc<UserApplicationService>,
    auth_service: Arc<AuthService>,
    config: Arc<AppConfig>,
}

impl ServiceContainer {
    pub async fn new(config: AppConfig) -> AppResult<Self> {
        let config = Arc::new(config);
        
        // Create database connection
        let db_pool = create_db_pool(&config.database).await?;
        
        // Create repositories
        let user_repo = Box::new(PostgresUserRepository::new(db_pool.clone()));
        let email_service = Box::new(SmtpEmailService::new(
            config.smtp.host.clone(),
            config.smtp.port,
            config.smtp.username.clone(),
            config.smtp.password.clone(),
        ));
        
        // Create domain services
        let user_domain_service = UserDomainService::new(user_repo, email_service);
        
        // Create application services
        let user_service = Arc::new(UserApplicationService::new(user_domain_service));
        let auth_service = Arc::new(AuthService::new(config.clone(), db_pool));
        
        Ok(ServiceContainer {
            user_service,
            auth_service,
            config,
        })
    }

    pub fn user_service(&self) -> Arc<UserApplicationService> {
        self.user_service.clone()
    }

    pub fn auth_service(&self) -> Arc<AuthService> {
        self.auth_service.clone()
    }

    pub fn config(&self) -> Arc<AppConfig> {
        self.config.clone()
    }
}

// Usage in main application
#[tokio::main]
async fn main() -> AppResult<()> {
    let config = AppConfig::load()?;
    let container = ServiceContainer::new(config).await?;
    
    // Start web server with container
    start_web_server(container).await?;
    
    Ok(())
}
```

**Architectural Best Practices:**
- Keep domain logic independent of infrastructure
- Use dependency injection for testability
- Design APIs with clear ownership semantics
- Prefer composition over inheritance
- Use type system to prevent invalid states
- Keep modules focused on single responsibilities
- Design for extensibility with traits and generics
- Use builder pattern for complex configurations
- Implement proper error boundaries between layers
