use tonic::{transport::Server, Request, Response, Status};
use user::user_service_server::{UserService, UserServiceServer};
use user::{UserRequest, UserResponse, GetUserRequest};
use sqlx::PgPool;
use dotenv::dotenv;
use std::env;
use std::sync::Arc;
use uuid::Uuid;
use chrono::NaiveDate;
use bcrypt::{hash, DEFAULT_COST};
mod init;
use init::generate_wallet;

pub mod user {
    tonic::include_proto!("user");
}

#[derive(Debug)]
pub struct MyUserService {
    pub db_pool: Arc<PgPool>,
}

#[tonic::async_trait]
impl UserService for MyUserService {
    async fn create_user(
        &self,
        request: Request<UserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let user = request.into_inner();
        let user_id = Uuid::new_v4();
        let (wallet_address, wallet_private_key) = generate_wallet();
        
        // Validate DOB format
        let dob = NaiveDate::parse_from_str(&user.dob, "%d-%m-%Y")
            .map_err(|_| Status::invalid_argument("Invalid date format. Use DD-MM-YYYY"))?;

        // Check if user already exists (by username, email, or mobile number)
        let existing_user = sqlx::query!(
            "SELECT id FROM users WHERE username = $1 OR email = $2 OR mobno = $3",
            user.username,
            user.email,
            user.mobno
        )
        .fetch_optional(&*self.db_pool)
        .await
        .map_err(|e| Status::internal(format!("Database error: {:?}", e)))?;
        if existing_user.is_some() {
            return Err(Status::already_exists("Username, Email, or Mobile Number already exists"));
        }

        let hashed_password = hash(user.password, DEFAULT_COST)
            .map_err(|_| Status::internal("Failed to hash password"))?;

        // Insert new user with hashed password
        let rec = sqlx::query!(
            "INSERT INTO users (id, username, email, password, dob, mobno, wallet_address, private_key) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
            user_id,
            user.username,
            user.email,
            hashed_password, 
            dob,
            user.mobno,
            wallet_address,
            wallet_private_key
        )
        .fetch_one(&*self.db_pool)
        .await
        .map_err(|e| Status::internal(format!("Failed to insert user: {:?}", e)))?;

        Ok(Response::new(UserResponse {
            id: rec.id.to_string(),
            username: user.username,
            email: user.email,
            password: hashed_password,
            dob: dob.to_string(),
            mobno: user.mobno,
            wallet_address: wallet_address,
            message: format!("User created with ID {}", rec.id),
        }))
    }

    // ✅ Retrieve User and Verify Password
    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let req = request.into_inner();

        let user_uuid = Uuid::parse_str(&req.id)
            .map_err(|_| Status::invalid_argument("Invalid UUID format"))?;

        let rec = sqlx::query!(
            "SELECT id, username, email, password, dob, mobno, wallet_address FROM users WHERE id = $1",
            user_uuid
        )
        .fetch_one(&*self.db_pool)
        .await
        .map_err(|e| {
            eprintln!("Database query error: {:?}", e);
            Status::not_found("User not found")
        })?;

        let response = UserResponse {
            id: rec.id.to_string(),
            username: rec.username,
            email: rec.email,
            password: rec.password,
            dob: rec.dob.to_string(),
            mobno: rec.mobno,
            wallet_address: rec.wallet_address.unwrap_or_default(),
            message: "User found".to_string(),
        };

        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    let pool = PgPool::connect(&database_url).await?;
    let db_pool = Arc::new(pool);

    let user_service = MyUserService { db_pool };

    let addr = "127.0.0.1:50051".parse()?; 
    println!("gRPC Server running on {}", addr);

    Server::builder()
        .add_service(UserServiceServer::new(user_service))
        .serve(addr)
        .await?;

    Ok(())
}
