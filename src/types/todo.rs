use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone)]
pub struct TodoStore {
    pub items: Arc<RwLock<Vec<Todo>>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Todo {
    pub id: usize,
    pub user_id: usize,
    pub title: String,
    pub completed: bool,
}

// impl Todo {
//     pub fn new(id: usize, user_id: usize, title: String) -> Self {
//         Self {
//             id,
//             user_id,
//             title,
//             completed: false,
//         }
//     }
// }
