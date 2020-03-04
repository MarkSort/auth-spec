use hyper::{Body, Method, Request, Response, StatusCode};

#[tokio::main]
async fn main() {
    let mut c = Checks::new("http://localhost:3000".into());

    c.path = "/users";

    let response = c.post_no_body("no body/content-length").await;
    c.check_error_response_multi(response, vec!["body", "content-length"]);

    let response = c.post_bad_content_type(
        "content-type other than null or application/json"
    ).await;
    c.check_error_response(response, "content-type");

    let response = c.post("can't parse json", "not json".into(), StatusCode::BAD_REQUEST).await;
    c.check_error_response(response, "parse");

    let email_1 = format!("test+{:0>8x}@example.com", rand::random::<u32>());
    let (json_response, _) = c.post(
        "correct response format",
        format!(r#"{{"email":"{}","password":"password"}}"#, email_1),
        StatusCode::OK,
    ).await;

    if let Some(json_response) = json_response {
        if let Some(email) = c.get_property_string(json_response, "email") {
            c.check(
                email == email_1,
                format!("expected email to be '{}' but got '{}'", email_1, email)
            );
        }
    } else {
        c.fail("response is not json".into());
    }


    println!("\n{} Passed / {} Failed", c.passed, c.failed);
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
    fn get_property_string(&mut self, json: serde_json::Value, name: &'static str) -> Option<String> {
        if let Some(property_value) = json.get(name) {
            if let Some(value) = property_value.as_str() {
                return Some(value.to_string())
            } else {
                self.fail(format!("json '{}' property is not a string", name));
            }
        } else {
            self.fail(format!("json does not have an '{}' property: {:?}", name, json));
        }
        None
    }

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

    async fn post_no_body(&mut self, group: &'static str) -> (Option<serde_json::Value>, Option<String>) {
        self.group = group;
        self.method = Method::POST;
        let response = self.client.request(
            Request::builder()
                .method(Method::POST)
                .uri(format!("{}{}", self.base_url, self.path))
                .body(Body::empty())
                .unwrap()
        ).await.unwrap();

        self.check_response(response, StatusCode::BAD_REQUEST).await
    }

    async fn post_bad_content_type(&mut self, group: &'static str) -> (Option<serde_json::Value>, Option<String>) {
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

        self.check_response(response, StatusCode::BAD_REQUEST).await
    }

    async fn post(&mut self, group: &'static str, body: String, expected_status: StatusCode) -> (Option<serde_json::Value>, Option<String>) {
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

        self.check_response(response, expected_status).await
    }

    async fn check_response(&mut self, response: Response<Body>, expected_status: StatusCode) -> (Option<serde_json::Value>, Option<String>) {
        let status = response.status();
        self.check(
            status == expected_status,
            format!("should have returned '{}' but returned '{}'", expected_status, status),
        );

        let json = if self.expect_json {
            self.check_json_content_type(response).await
        } else {
            None
        };

        if json.is_some() {
            (json, None)
        } else {
            (None, Some("body".into()))
        }
    }

    fn check_error_response(
        &mut self,
        response: (Option<serde_json::Value>, Option<String>),
        needle: &str,
    ) {
        self.check_error_response_multi(response, vec![needle])
    }


    fn check_error_response_multi(
        &mut self,
        response: (Option<serde_json::Value>, Option<String>),
        needles: Vec<&str>,
    ) {
        let (json_response, other_response) = response;
        if let Some(json_response) = json_response {
            if let Some(error) = self.get_property_string(json_response, "email") {
                self.check_contains_one("json 'error' property", error, needles);
            }
        } else if let Some(other_response) = other_response {
            self.check_contains_one("body", other_response, needles);
        } else {
            self.fail("no body in response".into());
        }
    }

    fn check_contains_one(&mut self, prefix: &str, haystack: String, needles: Vec<&str>) -> bool {
        let haystack = haystack.to_lowercase();
        if needles.len() == 1 {
            return self.check(
                haystack.contains(&needles[0]),
                format!("{} does not mention '{}'", prefix, needles[0])
            )
        }
        for needle in needles.clone() {
            if haystack.contains(&needle) {
                self.pass(1);
                return true
            }
        }
        self.fail(format!("{} does not mention one of: {}", prefix, needles.join(", ")));
        false
    }

    async fn check_json_content_type(&mut self, response: Response<Body>) -> Option<serde_json::Value> {
        match response.headers().get("content-type") {
            None => {
                self.fail("missing content-type".into());
                None
            }
            Some(content_type) => {
                self.pass(1);

                let content_type = content_type.to_str().unwrap();
                let content_type_parts: Vec<&str> = content_type.split(';').collect();

                if !self.check(
                    content_type_parts[0] == "application/json",
                    format!("content-type is '{}' instead of application/json", content_type),
                ) {
                    return None
                } else {
                    self.check(
                        content_type_parts.len() == 2 && content_type_parts[1] == "encoding=utf8",
                        "content-type missing 'encoding=utf8' or has too many parts".into(),
                    );
                }

                let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

                match serde_json::from_slice(body.to_vec().as_slice()) {
                    Ok(json) => {
                        self.pass(1);
                        Some(json)
                    }
                    Err(e) => {
                        self.fail("could not parse response as json".into());
                        println!("error: {:?}", e);
                        println!("body: {:?}", body);

                        None
                    }
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
