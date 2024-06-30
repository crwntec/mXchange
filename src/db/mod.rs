
use crate::models::{Mail, User};
use rusqlite::{params, Connection, Result};

fn get_connection() -> Connection {
    rusqlite::Connection::open("mail.db").expect("Could not open database")
}
pub fn init() {
    let conn = get_connection();
    conn.execute("
    CREATE TABLE IF NOT EXISTS mail (
        id INTEGER PRIMARY KEY,
        uid TEXT NOT NULL,
        sender TEXT NOT NULL,
        reciever TEXT NOT NULL,
        body TEXT NOT NULL
    )", []).unwrap();
    conn.execute("CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        address TEXT NOT NULL,
        password TEXT NOT NULL
    )", []).unwrap();
}
pub fn add_mail(mail: &Mail) -> Result<()> {
    let conn = get_connection();
    println!("Adding mail to database");
    conn.execute(
        "INSERT INTO mail (uid, sender, reciever, body) VALUES (?1, ?2, ?3, ?4)",
        (&mail.uid.to_string(), &mail.sender, &mail.reciever, &mail.body),
    )?;
    Ok(())
}

pub fn get_mailbox_size(user: String) -> Result<i32> {
    let conn = get_connection();
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM mail where reciever = ?1")?;
    let mut rows = stmt.query(params![user])?;
    let size = rows.next()?.unwrap().get(0)?;
    Ok(size)
}

pub fn get_mail(user: String) -> Result<Vec<Mail>> {
    let conn = get_connection();
    let mut stmt = conn.prepare("SELECT id, uid, sender, reciever, body FROM mail WHERE reciever = ?1")?;
    let mail_iter = stmt.query_map(params![user], |row| {
        Ok(Mail {
            uid: row.get(1)?,
            sender: row.get(2)?,
            reciever: row.get(3)?,
            body: row.get(4)?,
        })
    })?;
    let mut mails = Vec::new();
    for mail in mail_iter {
        mails.push(mail?);
    }
    Ok(mails)
}
pub fn delete_mail(id: i32) -> Result<()> {
    let conn = get_connection();
    conn.execute("DELETE FROM mail WHERE id=?1", params![id])?;
    Ok(())
}
pub fn get_user_by_address(address: String) -> Result<User> {
    let conn = get_connection();
    conn.query_row("SELECT name, address, password FROM users WHERE address = ?1", params![address], |row| {
        Ok(User {
            name: row.get(0).unwrap(),
            address: row.get(1).unwrap(),
            password: row.get(2).unwrap()
        })
    })
}