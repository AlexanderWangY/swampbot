use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RoleErrorResponse {
    pub error: String,
}

#[derive(Deserialize, Debug)]
pub struct RoleSuccessResponse {
    pub roles: Vec<String>,
}
