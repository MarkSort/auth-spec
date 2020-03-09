use hyper::{Body, Method, Request, Response, StatusCode};

pub struct Checker {
    pub passed: u16,
    pub failed: u16,
    group: &'static str,

    base_url: String,

    pub path: &'static str,
    method: Method,
    expect_json: bool,

    client: hyper::Client<hyper::client::HttpConnector, Body>,
}

impl Checker {
    pub fn get_property_string(&mut self, json: &serde_json::Value, name: &'static str) -> Option<String> {
        if let Some(property_value) = json.get(name) {
            if let Some(value) = property_value.as_str() {
                return Some(value.to_string())
            } else {
                self.fail(format!("json '{}' property is not a string", name));
            }
        } else {
            self.fail(format!("json does not have a '{}' property: {:?}", name, json));
        }
        None
    }

    pub fn get_property_i64(&mut self, json: &serde_json::Value, name: &'static str) -> Option<i64> {
        if let Some(property_value) = json.get(name) {
            if let Some(value) = property_value.as_i64() {
                return Some(value)
            } else {
                self.fail(format!("json '{}' property is not an integer", name));
            }
        } else {
            self.fail(format!("json does not have a '{}' property: {:?}", name, json));
        }
        None
    }

    pub fn check(&mut self, check: bool, description: String) -> bool {
        if check {
            self.pass(1);
        } else {
            self.fail(description);
        }
        check
    }

    pub fn pass(&mut self, count: u16) {
        self.passed = self.passed + count;
    }

    pub fn fail(&mut self, description: String) {
        self.failed = self.failed + 1;
        println!("Failed: {} {} - {} - {}", self.method, self.path, self.group, description);
    }

    pub async fn get(&mut self, group: &'static str, expected_status: StatusCode) -> (Option<serde_json::Value>, Option<String>) {
        self.group = group;
        self.method = Method::GET;
        let response = self.client.request(
            Request::builder()
                .method(Method::GET)
                .uri(format!("{}{}", self.base_url, self.path))
                .body(Body::empty())
                .unwrap()
        ).await.unwrap();

        self.check_response(response, expected_status).await
    }

    pub async fn get_with_token(&mut self, group: &'static str, token_secret: String, expected_status: StatusCode) -> (Option<serde_json::Value>, Option<String>) {
        self.group = group;
        self.method = Method::GET;
        let response = self.client.request(
            Request::builder()
                .method(Method::GET)
                .uri(format!("{}{}", self.base_url, self.path))
                .header("cookie", format!("token={}", token_secret))
                .body(Body::empty())
                .unwrap()
        ).await.unwrap();

        self.check_response(response, expected_status).await
    }

    pub async fn delete(&mut self, group: &'static str, expected_status: StatusCode) -> (Option<serde_json::Value>, Option<String>) {
        self.group = group;
        self.method = Method::DELETE;
        let response = self.client.request(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!("{}{}", self.base_url, self.path))
                .body(Body::empty())
                .unwrap()
        ).await.unwrap();

        self.check_response(response, expected_status).await
    }

    pub async fn delete_with_token(&mut self, group: &'static str, token_secret: String, expected_status: StatusCode) -> (Option<serde_json::Value>, Option<String>) {
        self.group = group;
        self.method = Method::DELETE;
        let response = self.client.request(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!("{}{}", self.base_url, self.path))
                .header("cookie", format!("token={}", token_secret))
                .body(Body::empty())
                .unwrap()
        ).await.unwrap();

        self.check_response(response, expected_status).await
    }

    pub async fn post_no_body(&mut self, group: &'static str, expected_status: StatusCode) -> (Option<serde_json::Value>, Option<String>) {
        self.group = group;
        self.method = Method::POST;
        let response = self.client.request(
            Request::builder()
                .method(Method::POST)
                .uri(format!("{}{}", self.base_url, self.path))
                .body(Body::empty())
                .unwrap()
        ).await.unwrap();

        self.check_response(response, expected_status).await
    }

    pub async fn post_with_token(&mut self, group: &'static str, token_secret: String, expected_status: StatusCode) -> (Option<serde_json::Value>, Option<String>) {
        self.group = group;
        self.method = Method::POST;
        let response = self.client.request(
            Request::builder()
                .method(Method::POST)
                .uri(format!("{}{}", self.base_url, self.path))
                .header("cookie", format!("token={}", token_secret))
                .body(Body::empty())
                .unwrap()
        ).await.unwrap();

        self.check_response(response, expected_status).await
    }

    pub async fn post_bad_content_type(&mut self, group: &'static str, expected_status: StatusCode) -> (Option<serde_json::Value>, Option<String>) {
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

        self.check_response(response, expected_status).await
    }

    pub async fn post(&mut self, group: &'static str, body: String, expected_status: StatusCode) -> (Option<serde_json::Value>, Option<String>) {
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

    pub async fn post_content_type(&mut self, group: &'static str, content_type: &'static str, body: String, expected_status: StatusCode) -> (Option<serde_json::Value>, Option<String>) {
        self.group = group;
        self.method = Method::POST;
        let response = self.client.request(
            Request::builder()
                .method(Method::POST)
                .uri(format!("{}{}", self.base_url, self.path))
                .header("content-type", content_type)
                .body(Body::from(body))
                .unwrap()
        ).await.unwrap();

        self.check_response(response, expected_status).await
    }

    pub async fn post_no_content_type(&mut self, group: &'static str, body: String, expected_status: StatusCode) -> (Option<serde_json::Value>, Option<String>) {
        self.group = group;
        self.method = Method::POST;
        let response = self.client.request(
            Request::builder()
                .method(Method::POST)
                .uri(format!("{}{}", self.base_url, self.path))
                .body(Body::from(body))
                .unwrap()
        ).await.unwrap();

        self.check_response(response, expected_status).await
    }

    pub async fn check_response(&mut self, response: Response<Body>, expected_status: StatusCode) -> (Option<serde_json::Value>, Option<String>) {
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

    pub fn check_error_response(
        &mut self,
        response: (Option<serde_json::Value>, Option<String>),
        needle: &str,
    ) {
        self.check_error_response_multi(response, vec![needle])
    }


    pub fn check_error_response_multi(
        &mut self,
        response: (Option<serde_json::Value>, Option<String>),
        needles: Vec<&str>,
    ) {
        let (json_response, other_response) = response;
        if let Some(json_response) = json_response {
            if let Some(error) = self.get_property_string(&json_response, "error") {
                self.check_contains_one("json 'error' property", error, needles);
            }
        } else if let Some(other_response) = other_response {
            self.check_contains_one("body", other_response, needles);
        } else {
            self.fail("no body in response".into());
        }
    }

    pub fn check_contains_one(&mut self, prefix: &str, haystack: String, needles: Vec<&str>) -> bool {
        let haystack = haystack.to_lowercase();
        if needles.len() == 1 {
            return self.check(
                haystack.contains(&needles[0]),
                format!("{} does not mention '{}': '{}'", prefix, needles[0], haystack)
            )
        }
        for needle in needles.clone() {
            if haystack.contains(&needle) {
                self.pass(1);
                return true
            }
        }
        self.fail(format!("{} does not mention one of '{}': '{}'", prefix, needles.join(", "), haystack));
        false
    }

    pub async fn check_json_content_type(&mut self, response: Response<Body>) -> Option<serde_json::Value> {
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
                        content_type_parts.len() == 2 && content_type_parts[1] == "charset=utf-8",
                        "content-type missing 'charset=utf-8' or has too many parts".into(),
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


    pub fn new(base_url: String) -> Checker {
        Checker {
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
