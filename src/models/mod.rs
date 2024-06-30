#[derive(Debug)]
pub struct Mail {
    pub uid: String,
    pub sender: String,
    pub reciever: String,
    pub body: String,
}

pub struct User {
    pub name: String,
    pub address: String,
    pub password: String,
}