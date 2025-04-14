use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct UserIdCause {
    #[serde(rename = "userId")]
    pub user_id: String,
}

impl UserIdCause {
    pub fn is_mine(&self, my_user_id: &str) -> bool {
        self.user_id == my_user_id
    }
}
