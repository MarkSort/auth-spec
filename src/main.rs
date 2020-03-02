use hyper::{Body, Method, Request, Response, StatusCode};

#[tokio::main]
async fn main() {
    let mut c = Checks::new("http://localhost:8000".into());

    c.path = "/users";

    c.post_no_body("no content-length").await;

    c.post_bad_content_type("content-type other than null or application/json").await;

    c.post("can't parse json", "not json".into(), StatusCode::BAD_REQUEST).await;

    c.post(
        "correct response format",
        r#"{"email":"test1234@example.com","password":"password"}"#.into(),
        StatusCode::OK,
    ).await;

    println!("{} Passed / {} Failed", c.passed, c.failed);
}

struct Checks {
    passed: u16,
    failed: u16,
    group: &'static str,

    base_url: String,

    path: &'static str,
    method: Method,
    expect_json: bool,

    client: hyper::Client<hyper::client::HttpConnector, Body>,
}

impl Checks {
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
        println!("Failed: {} {} - {} - {}", self.method, self.path, self.group, description);
    }

    async fn post_no_body(&mut self, group: &'static str) {
        self.group = group;
        self.method = Method::POST;
        let response = self.client.request(
            Request::builder()
                .method(Method::POST)
                .uri(format!("{}{}", self.base_url, self.path))
                .body(Body::empty())
                .unwrap()
        ).await.unwrap();

        self.check_response(response, StatusCode::BAD_REQUEST);
    }

    async fn post_bad_content_type(&mut self, group: &'static str) {
        self.group = group;
        self.method = Method::POST;
        let response = self.client.request(
            Request::builder()
                .method(Method::POST)
                .uri(format!("{}{}", self.base_url, self.path))
                .header("content-type", "image/png")
                .body(Body::empty())
                .unwrap()
        ).await.unwrap();

        self.check_response(response, StatusCode::BAD_REQUEST);
    }

    async fn post(&mut self, group: &'static str, body: String, expected_status: StatusCode) {
        self.group = group;
        self.method = Method::POST;
        let response = self.client.request(
            Request::builder()
                .method(Method::POST)
                .uri(format!("{}{}", self.base_url, self.path))
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap()
        ).await.unwrap();

        self.check_response(response, expected_status);
    }

    fn check_response(&mut self, response: Response<Body>, expected_status: StatusCode) {
        let status = response.status();
        self.check(
            status == expected_status,
            format!("should have returned 400 but returned {}", status),
        );

        if self.expect_json {
            self.check_json_content_type(response);
        }
    }

    fn check_json_content_type(&mut self, response: Response<Body>) {
        match response.headers().get("content-type") {
            None => {
                self.fail("missing content-type".into());
            }
            Some(content_type) => {
                self.pass(1);

                let content_type = content_type.to_str().unwrap();
                let content_type_parts: Vec<&str> = content_type.split(';').collect();

                if self.check(
                    content_type_parts[0] == "application/json",
                    format!("content-type is '{}' instead of application/json", content_type),
                ) {
                    self.check(
                        content_type_parts.len() == 2 && content_type_parts[1] == "encoding=utf8",
                        "content-type missing 'encoding=utf8' or has too many parts".into(),
                    );
                }
            }
        }
    }


    fn new(base_url: String) -> Checks {
        Checks {
            passed: 0,
            failed: 0,
            group: "",

            base_url,
            path: "/",
            method: Method::GET,
            expect_json: true,

            client: hyper::Client::new(),
        }
    }
}
