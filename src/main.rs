mod checker;
mod checks {
    pub mod tokens;
    pub mod tokenscurrent;
    pub mod tokenscurrentrefresh;
    pub mod users;
}

#[tokio::main]
async fn main() {
    let mut c = checker::Checker::new("http://localhost:3000".into());

    checks::tokens::check(&mut c).await;
    checks::tokenscurrent::check(&mut c).await;
    checks::tokenscurrentrefresh::check(&mut c).await;
    checks::users::check(&mut c).await;

    println!("\n{} Passed / {} Failed", c.passed, c.failed);
}

