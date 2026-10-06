// The Model derive expands to `bee_orm::…` paths, so the re-export must be
// in scope at the invocation site (bee_orm is not a direct dependency).
use bee_rust::bee_orm;

use bee_rust::bee_orm::Model;

#[derive(Model)]
#[bee(table = "tags")]
pub struct Tag {
    #[bee(pk, auto)]
    pub id: i64,
    pub name: String,
}

#[derive(Model)]
#[bee(table = "links")]
#[bee(m2m(Tag))]
pub struct Link {
    #[bee(pk, auto)]
    pub id: i64,
    pub code: String,
    pub url: String,
    #[bee(auto_now_add)]
    pub created_at: i64,
    #[bee(soft_delete)]
    pub deleted: bool,
}

#[derive(Model)]
#[bee(table = "clicks")]
pub struct Click {
    #[bee(pk, auto)]
    pub id: i64,
    #[bee(fk = Link)]
    pub link_id: i64,
    #[bee(auto_now_add)]
    pub created_at: i64,
}
