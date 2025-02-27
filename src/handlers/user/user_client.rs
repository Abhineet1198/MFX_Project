use actix_web::{web, App, HttpServer, Responder, HttpResponse, post, get};
use serde::{Deserialize, Serialize};
use user::user_service_client::UserServiceClient;
use user::{UserRequest, GetUserRequest};

pub mod user {
    tonic::include_proto!("user");
}

// Struct for HTTP response serialization (Removed `password` field)
#[derive(Debug, Serialize)]
struct SerializableUserResponse {
    id: String,
    username: String,
    email: String,
    password: String,
    dob: String,
    mobno: String,
    wallet_address: String,
    message: String
}

// Struct for HTTP request body (Password is still required for creating user)
#[derive(Debug,Deserialize)]
struct UserInput {
    username: String,
    email: String,
    password: String,
    dob: String,
    mobno: String,
}

// gRPC Client Function to create a user
async fn send_to_grpc(user: UserInput) -> Result<SerializableUserResponse, Box<dyn std::error::Error>> {
    let mut client = UserServiceClient::connect("http://127.0.0.1:50051").await?;

    let request = tonic::Request::new(UserRequest {
        username: user.username.clone(),
        email: user.email.clone(),
        password: user.password, // Sent but not stored in response
        dob: user.dob.clone(),
        mobno: user.mobno.clone(),
    });

    let response = client.create_user(request).await?.into_inner();

    let serialized_response = SerializableUserResponse {
        message: "User registered successfully!".to_string(), 
        id: response.id,
        username: response.username,
        email: response.email,
        password: response.password,
        dob: response.dob,
        mobno: response.mobno,
        wallet_address: response.wallet_address,
    };

    Ok(serialized_response)
}

// gRPC Client Function to get a user by ID
async fn get_user_from_grpc(user_id: String) -> Result<SerializableUserResponse, Box<dyn std::error::Error>> {
    let mut client = UserServiceClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(GetUserRequest { id: user_id });

    let response = client.get_user(request).await?.into_inner();

    let serialized_response = SerializableUserResponse {
        message: "User found".to_string(),
        id: response.id,
        username: response.username,
        email: response.email,
        password: response.password,
        dob: response.dob,
        mobno: response.mobno,
        wallet_address: response.wallet_address
    };

    Ok(serialized_response)
}

// HTTP POST Handler to create a user
#[post("/create-user")]
async fn create_user(user: web::Json<UserInput>) -> impl Responder {
    match send_to_grpc(user.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            let error_message = e.to_string();
            if error_message.contains("already exists") {
                HttpResponse::Conflict().body("User with this username,email or mobno already exists")
            } else {
                HttpResponse::InternalServerError().body("Failed to process request")
            }
        }
    }
}

// HTTP GET Handler to fetch a user by ID
#[get("/get-user/{id}")]
async fn get_user(path: web::Path<String>) -> impl Responder {
    let user_id = path.into_inner();
    
    match get_user_from_grpc(user_id).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(_) => HttpResponse::NotFound().body("User not found"),
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let server_address = "127.0.0.1:8081";

    println!("Client (HTTP) running on {}", server_address);

    HttpServer::new(|| {
        App::new()
            .service(create_user) // Register HTTP POST endpoint
            .service(get_user)    // Register HTTP GET endpoint
    })
    .bind(server_address)?
    .run()
    .await
}
