use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;
use warp::http::StatusCode;
use serde::Deserialize;

// Test the server with curl with the following command
//  curl -d '{"username":"AbsoluteVirtue", "password":"password123"}' -H "Content-Type: application/json" -X POST http://localhost:3030/register -v

#[derive(Deserialize)]
struct User {
    username: String,
    password: String,
}

#[tokio::main]
async fn main() {
    let db = Arc::new(Mutex::new(HashMap::<String, User>::new()));
    let db = warp::any().map(move || Arc::clone(&db));

    let register = warp::post()
        .and(warp::path("register"))
        .and(warp::body::json())
        .and(db.clone())
        .and_then(register);
    let login = warp::post()
        .and(warp::path("login"))
        .and(warp::body::json())
        .and(db.clone())
        .and_then(login);

    let routes = register.or(login);
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;

    //let routes = warp::path("counter").and(db).and_then(counter);

    //warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn register(new_user: User, db: Arc<Mutex<HashMap<String, User>>>) -> Result<impl warp::Reply, warp::Rejection>{
    let mut users = db.lock().await;
    if users.contains_key(&new_user.username) {
        println!("user already exist");
        return Ok(StatusCode::BAD_REQUEST)
    }
    users.insert(new_user.username.clone(), new_user);
    println!("register sucess");
    Ok(StatusCode::CREATED)
}

async fn login(credentials: User, db: Arc<Mutex<HashMap<String, User>>>) -> Result<impl warp::Reply, warp::Rejection> {
    let users = db.lock().await;
    match users.get(&credentials.username) {
        None => {
            println!("user doesn't exist");
            Ok(StatusCode::BAD_REQUEST)
        },
        Some(user) => {
            if credentials.password == user.password {
                println!("login success");
                Ok(StatusCode::OK)
            } else {
                println!("login failed");
                Ok(StatusCode::UNAUTHORIZED)
            }
        }
    }
}