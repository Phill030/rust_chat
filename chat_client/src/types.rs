#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum ServerProtocol {
    BroadcastMessage { sender: String, content: String },
    AuthenticateToken { token: String },
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum ClientProtocol {
    SendMessage { hwid: String, content: String },
    ChangeUsername { hwid: String, new_username: String },
    RequestAuthentication { hwid: String },
}
