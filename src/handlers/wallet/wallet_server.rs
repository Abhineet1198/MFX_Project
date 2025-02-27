use sqlx::PgPool;
use tonic::{Request, Response, Status};
use wallet::wallet_service_server::WalletService;
use wallet::{WalletRequest, WalletResponse};
use uuid::Uuid;

pub mod wallet {
    tonic::include_proto!("wallet");
}

#[derive(Debug, Clone)]
pub struct MyWalletService {
    db: PgPool,
}

impl MyWalletService {
    pub fn new(db: PgPool) -> Self {
        MyWalletService { db }
    }
}

#[tonic::async_trait]
impl WalletService for MyWalletService {
    async fn create_wallet(&self, request: Request<WalletRequest>) -> Result<Response<WalletResponse>, Status> {
        let req = request.into_inner();
        let wallet_id = Uuid::new_v4().to_string();

        let query = sqlx::query!(
            "INSERT INTO wallets (id, user_id, balance) VALUES ($1, $2, $3)",
            wallet_id, req.user_id, req.amount
        )
        .execute(&self.db)
        .await;

        match query {
            Ok(_) => Ok(Response::new(WalletResponse {
                id: wallet_id,
                user_id: req.user_id,
                balance: req.amount,
            })),
            Err(_) => Err(Status::internal("Failed to insert wallet into database")),
        }
    }
}
