use hyper::{Body, Method, Request, Response, StatusCode};

#[tokio::main]
async fn main() {
    let mut c = Checks::new();

    let client = hyper::Client::new();

    c.group = "POST /users 400 - no content-length";
    let response = client.request(
        Request::builder()
            .method(Method::POST)
            .uri("http://localhost:3000/users")
            //.header("content-type", "application/json")
            //.body(Body::from(r#"{"library":"hyper"}"#))
            .body(Body::empty())
            .unwrap()
    ).await.unwrap();

    let status = response.status();
    c.check(
        status == StatusCode::BAD_REQUEST,
        format!("should have returned 400 but returned {}", status),
    );
    let (passed, failed) = check_content_type(response);
    c.bulk(passed, failed);

    
    c.group = "POST /users 400 - content-type other than null or application/json";
    let response = client.request(
        Request::builder()
            .method(Method::POST)
            .uri("http://localhost:3000/users")
            .header("content-type", "image/png")
            .body(Body::empty())
            .unwrap()
    ).await.unwrap();

    let status = response.status();
    c.check(
        status == StatusCode::BAD_REQUEST,
        format!("should have returned 400 but returned {}", status),
    );
    let (passed, failed) = check_content_type(response);
    c.bulk(passed, failed);


    c.group = "POST /users 400 - can't parse json";
    let response = client.request(
        Request::builder()
            .method(Method::POST)
            .uri("http://localhost:3000/users")
            .header("content-type", "application/json")
            .body(Body::from("not json"))
            .unwrap()
    ).await.unwrap();

    let status = response.status();
    c.check(
        status == StatusCode::BAD_REQUEST,
        format!("should have returned 400 but returned {}", status),
    );
    let (passed, failed) = check_content_type(response);
    c.bulk(passed, failed);


    c.group = "POST /users 200 - correct response format";
    let response = client.request(
        Request::builder()
            .method(Method::POST)
            .uri("http://localhost:3000/users")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"email":"test1234@example.com","password":"password"}"#))
            .unwrap()
    ).await.unwrap();

    let status = response.status();
    c.check(
        status == StatusCode::OK,
        format!("should have returned 200 but returned {}", status),
    );
    let (passed, failed) = check_content_type(response);
    c.bulk(passed, failed);

    println!("{} Passed / {} Failed", c.passed, c.failed);
}

fn check_content_type(response: Response<Body>) -> (u16, Vec<String>) {
    let mut passed = 0;
    let mut failed: Vec<String> = vec!();
    match response.headers().get("content-type") {
        None => {
            failed.push("missing content-type".into())
        }
        Some(content_type) => {
            passed = passed + 1;

            let content_type = content_type.to_str().unwrap();
            let content_type_parts: Vec<&str> = content_type.split(';').collect();
            if content_type_parts[0] == "application/json" {
                passed = passed + 1;
                if content_type_parts.len() == 2 && content_type_parts[1] == "encoding=utf8" {
                    passed = passed + 1;
                } else {
                    failed.push("content-type missing 'encoding=utf8' or has too many parts".into());
                }
            } else {
                failed.push(format!("content-type is '{}' instead of application/json", content_type));
            }
        }
    }
    (passed, failed)
}

struct Checks {
    passed: u16,
    failed: u16,
    group: &'static str,
}

impl Checks {
    /*
    async fn sendRequest(&mut self, method: Method, path: String, group: &'static str) -> Response<Body> {
        self.group = format!("{} {} {}", method, path, group);
        self.client.request(
            Request::builder()
                .method(method)
                .uri(format!("http://localhost:3000{}", path))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"email":"test1234@example.com","password":"password"}"#))
                .unwrap()
        ).await.unwrap()
    }
    */

    fn check(&mut self, check: bool, description: String) -> bool {
        if check {
            self.pass(1);
        } else {
            self.fail(description);
        }
        check
    }

    fn pass(&mut self, count: u16) {
        self.passed = self.passed + count;
    }

    fn fail(&mut self, description: String) {
        self.failed = self.failed + 1;
        println!("Failed: {} - {}", self.group, description);
    }

    fn bulk(&mut self, passed: u16, failed: Vec<String>) {
        self.pass(passed);
        for error in failed {
            self.fail(error);
        }
    }

    fn new() -> Checks {
        Checks { passed: 0, failed: 0, group: "" }
    }
}

