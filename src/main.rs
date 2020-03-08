mod checker;
mod checks {
    pub mod users;
}

#[tokio::main]
async fn main() {
    let mut c = checker::Checker::new("http://localhost:3000".into());

    checks::users::check_users(&mut c).await;

    println!("\n{} Passed / {} Failed", c.passed, c.failed);
}

