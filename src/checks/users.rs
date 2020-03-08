use hyper::{StatusCode};

pub async fn check_users(c: &mut crate::checker::Checker) {
    c.path = "/users";

    // error cases
    let response = c.post_no_body("no body/content-length").await;
    c.check_error_response_multi(response, vec!["body", "content-length"]);

    let response = c.post_bad_content_type(
        "content-type other than null or application/json"
    ).await;
    c.check_error_response(response, "content-type");

    let response = c.post("can't parse json", "not json".into(), StatusCode::BAD_REQUEST).await;
    c.check_error_response(response, "parse");

    let response = c.post( "missing email", r#"{}"#.into(), StatusCode::BAD_REQUEST).await;
    c.check_error_response(response, "email");

    let response = c.post( "email must be string", r#"{"email":123}"#.into(), StatusCode::BAD_REQUEST).await;
    c.check_error_response(response, "string");

    let mut long_email: String = "".into();
    for _ in 0..14 {
        long_email = long_email + "01234567890";
    }
    long_email = long_email + "@example.com";
    let response = c.post("email too long", format!(r#"{{"email":"{}"}}"#, long_email), StatusCode::BAD_REQUEST).await;
    c.check_error_response(response, "150");

    let email_1 = format!("test+{:0>8x}@example.com", rand::random::<u32>());

    let response = c.post("missing password", format!(r#"{{"email":"{}"}}"#, email_1), StatusCode::BAD_REQUEST).await;
    c.check_error_response(response, "password");

    let response = c.post(
        "password must be string",
        format!(r#"{{ "email": "{}", "password": 123 }}"#, email_1),
        StatusCode::BAD_REQUEST
    ).await;
    c.check_error_response(response, "string");

    c.post(
        "identity with given email already exists - create",
        format!(r#"{{"email":"{}","password":"password"}}"#, email_1),
        StatusCode::OK,
    ).await;
    let response = c.post(
        "identity with given email already exists - check",
        format!(r#"{{"email":"{}","password":"password"}}"#, email_1),
        StatusCode::BAD_REQUEST
    ).await;
    c.check_error_response(response, "in use");

    let response = c.get("method not allowed", StatusCode::METHOD_NOT_ALLOWED).await;
    c.check_error_response(response, "method");


    // success cases
    let email_1 = format!("test+{:0>8x}@example.com", rand::random::<u32>());
    let (json_response, _) = c.post(
        "correct response format",
        format!(r#"{{"email":"{}","password":"password"}}"#, email_1),
        StatusCode::OK,
    ).await;

    if let Some(json_response) = json_response {
        if let Some(email) = c.get_property_string(json_response.clone(), "email") {
            c.check(
                email == email_1,
                format!("expected email to be '{}' but got '{}'", email_1, email)
            );
        }
        c.get_property_i64(json_response, "id");
    } else {
        c.fail("response is not json".into());
    }

    let email_1 = format!("test+{:0>8x}@example.com", rand::random::<u32>());
    c.post_content_type(
        "correct response format",
        "application/json",
        format!(r#"{{"email":"{}","password":"password"}}"#, email_1),
        StatusCode::OK,
    ).await;

    let email_1 = format!("test+{:0>8x}@example.com", rand::random::<u32>());
    c.post_content_type(
        "correct response format",
        "application/json;encoding=utf8",
        format!(r#"{{"email":"{}","password":"password"}}"#, email_1),
        StatusCode::OK,
    ).await;

    let email_1 = format!("test+{:0>8x}@example.com", rand::random::<u32>());
    c.post_no_content_type(
        "correct response format",
        format!(r#"{{"email":"{}","password":"password"}}"#, email_1),
        StatusCode::OK,
    ).await;
}
