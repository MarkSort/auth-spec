mod checker;
mod checks {
    pub mod users;
    pub mod tokens;
}

#[tokio::main]
async fn main() {
    let mut c = checker::Checker::new("http://localhost:3000".into());

    checks::users::check_users(&mut c).await;
    checks::tokens::check_tokens(&mut c).await;

    println!("\n{} Passed / {} Failed", c.passed, c.failed);
}

