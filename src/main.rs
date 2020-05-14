use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;
use warp::http::StatusCode;
use serde::{Deserialize, Serialize};

// Test the server with curl with the following command
//  curl -d '{"username":"AbsoluteVirtue", "password":"password123"}' -H "Content-Type: application/json" -X POST http://localhost:3030/register -v

#[derive(Deserialize)]
struct User {
    username: String,
    password: String,
}

#[derive(Deserialize, Serialize, Clone)]
struct Message {
    message: String,
}

impl Message {
    fn new(message: String) -> Message {
        Message{message}
    }
}

#[tokio::main]
async fn main() {

    //User table
    let db_user = Arc::new(Mutex::new(HashMap::<String, User>::new()));
    let db_user = warp::any().map(move || Arc::clone(&db_user));


    //Message table
    let mut messages = Vec::<Message>::new();
    messages.push(Message::new(String::from("I am secret 0")));
    messages.push(Message::new(String::from("I am secret 1")));
    messages.push(Message::new(String::from("I am secret 2")));
    let db_messages = Arc::new(Mutex::new(messages));
    let db_messages = warp::any().map(move || Arc::clone(&db_messages));


    let register = warp::post()
        .and(warp::path("register"))
        .and(warp::body::json())
        .and(db_user.clone())
        .and_then(register);

    let login = warp::post()
        .and(warp::path("login"))
        .and(warp::body::json())
        .and(db_user.clone())
        .and_then(login);

    let get_message = warp::get()
        .and(warp::path("get_message"))
        .and(warp::path::param::<usize>())
        .and(db_messages.clone())
        .and_then(read_message)
        .map(|message| {
            warp::reply::json(&message)
        });

    let write_message = warp::post()
        .and(warp::path("write_message"))
        .and(warp::body::json())
        .and(db_messages.clone())
        .and_then(write_message)
        .map(|_| {
            StatusCode::OK
        });

    let routes = register.or(login).or(get_message).or(write_message);
    warp::serve(routes).run(([192, 168, 0, 10], 3030)).await;

    //let routes = warp::path("counter").and(db).and_then(counter);

    //warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

// register should call login
async fn register(new_user: User, db_users: Arc<Mutex<HashMap<String, User>>>) -> Result<impl warp::Reply, warp::Rejection>{
    let mut users = db_users.lock().await;
    if users.contains_key(&new_user.username) {
        return Ok(warp::reply::with_status("Login already exists", StatusCode::BAD_REQUEST))
    }
    users.insert(new_user.username.clone(), new_user);
    Ok(warp::reply::with_status("User created",StatusCode::CREATED))
}

async fn login(credentials: User, db_users: Arc<Mutex<HashMap<String, User>>>) -> Result<impl warp::Reply, warp::Rejection> {
    let users = db_users.lock().await;
    match users.get(&credentials.username) {
        None => {
            Ok(warp::reply::with_status("User doesn't exist",StatusCode::BAD_REQUEST))
        },
        Some(user) => {
            if credentials.password == user.password {
                Ok(warp::reply::with_status("User logged in", StatusCode::OK))
            } else {
                Ok(warp::reply::with_status("Bad password", StatusCode::UNAUTHORIZED))
            }
        }
    }
}

async fn read_message(index: usize, db_messages: Arc<Mutex<Vec<Message>>>) -> Result<Message, warp::Rejection> {
    let messages = db_messages.lock().await;
    match messages.get(index) {
        None => Err(warp::reject::not_found()),
        Some(message) => {
            Ok(message.clone())
        }
    }
}

async fn write_message(message: Message, db_messages: Arc::<Mutex<Vec<Message>>>) -> Result<(), warp::Rejection> {
    let mut messages = db_messages.lock().await;
    messages.push(message);
    Ok(())
}
