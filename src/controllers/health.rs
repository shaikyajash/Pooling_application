use axum::{Json, http::StatusCode};
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
    status:String,
    message:String,

}


pub async fn health_get() -> (StatusCode, Json<HealthResponse>){
    let response = HealthResponse {
        status: "OK".to_string(),
        message: "Service is running".to_string(),
    };
    (StatusCode::OK, Json(response))
}