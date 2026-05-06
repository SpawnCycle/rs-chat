use std::collections::HashMap;

use chat_lib::types::{Message, User};
use uuid::Uuid;

pub trait UserLocator {
    fn get_user(&'_ self, id: Uuid) -> Option<&'_ User>;
}

pub trait MessageTrait {
    fn get_author_from<'a>(&self, users: &'a impl UserLocator) -> Option<&'a User>;
}

impl MessageTrait for Message {
    fn get_author_from<'a>(&self, users: &'a impl UserLocator) -> Option<&'a User> {
        users.get_user(*self.get_author())
    }
}

impl UserLocator for &[User] {
    fn get_user(&self, id: Uuid) -> Option<&User> {
        self.iter().find(|u| *u.get_id() == id)
    }
}

impl UserLocator for Vec<User> {
    fn get_user(&self, id: Uuid) -> Option<&User> {
        self.iter().find(|u| *u.get_id() == id)
    }
}

impl<S: std::hash::BuildHasher> UserLocator for HashMap<Uuid, User, S> {
    fn get_user(&self, id: Uuid) -> Option<&User> {
        self.get(&id)
    }
}
