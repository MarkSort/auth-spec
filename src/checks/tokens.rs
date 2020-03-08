use hyper::StatusCode;

pub async fn check_tokens(c: &mut crate::checker::Checker) {
    c.path = "/tokens";

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

    let response = c.post(
        "missing lifetime",
        format!(r#"{{"email":"{}","password":"password"}}"#, email_1),
        StatusCode::BAD_REQUEST
    ).await;
    c.check_error_response(response, "lifetime");

    let response = c.post(
        "lifetime must be string",
        format!(r#"{{"email":"{}","password":"password", "lifetime": 123 }}"#, email_1),
        StatusCode::BAD_REQUEST
    ).await;
    c.check_error_response(response, "no-expiration");

    let response = c.post(
        "invalid lifetime",
        format!(r#"{{"email":"{}","password":"password", "lifetime": "wrong" }}"#, email_1),
        StatusCode::BAD_REQUEST
    ).await;
    c.check_error_response(response, "no-expiration");

    let response = c.post(
        "invalid credentials; unknown email",
        format!(r#"{{"email":"{}", "password": "password", "lifetime": "no-expiration" }}"#, email_1),
        StatusCode::BAD_REQUEST
    ).await;
    c.check_error_response(response, "invalid");

    c.path = "/users";
    c.post(
        "invalid credentials; wrong password; create",
        format!(r#"{{"email":"{}","password":"otherpass"}}"#, email_1),
        StatusCode::OK,
    ).await;
    c.path = "/tokens";
    let response = c.post(
        "invalid credentials; wrong password; check",
        format!(r#"{{"email":"{}", "password": "password", "lifetime": "no-expiration" }}"#, email_1),
        StatusCode::BAD_REQUEST
    ).await;
    c.check_error_response(response, "invalid");

    let response = c.delete("method not allowed", StatusCode::METHOD_NOT_ALLOWED).await;
    c.check_error_response(response, "method");

    // success cases
    let email_1 = format!("test+{:0>8x}@example.com", rand::random::<u32>());
    c.path = "/users";
    c.post(
        "correct response format; create",
        format!(r#"{{"email":"{}","password":"password"}}"#, email_1),
        StatusCode::OK,
    ).await;
    c.path = "/tokens";
    let (json_response, _) = c.post(
        "correct response format; check",
        format!(r#"{{"email":"{}", "password": "password", "lifetime": "no-expiration" }}"#, email_1),
        StatusCode::OK
    ).await;
    if let Some(json_response) = json_response {
        c.get_property_string(json_response.clone(), "id");
        c.get_property_string(json_response.clone(), "secret");
        c.get_property_string(json_response.clone(), "lifetime");
        c.get_property_i64(json_response.clone(), "created");
        c.get_property_i64(json_response.clone(), "last_active");
    } else {
        c.fail("response is not json".into());
    }

    c.post(
        "lifetime: no-expiration",
        format!(r#"{{"email":"{}", "password": "password", "lifetime": "no-expiration" }}"#, email_1),
        StatusCode::OK
    ).await;

    c.post(
        "lifetime: remember-me",
        format!(r#"{{"email":"{}", "password": "password", "lifetime": "remember-me" }}"#, email_1),
        StatusCode::OK
    ).await;

    c.post(
        "lifetime: until-idle",
        format!(r#"{{"email":"{}", "password": "password", "lifetime": "until-idle" }}"#, email_1),
        StatusCode::OK
    ).await;

    c.post_content_type(
        "content-type: application/json",
        "application/json",
        format!(r#"{{"email":"{}", "password": "password", "lifetime": "no-expiration" }}"#, email_1),
        StatusCode::OK
    ).await;

    c.post_content_type(
        "content-type: application/json;charset=utf-8",
        "application/json;charset=utf-8",
        format!(r#"{{"email":"{}", "password": "password", "lifetime": "no-expiration" }}"#, email_1),
        StatusCode::OK
    ).await;

    c.post_no_content_type(
        "no content-type",
        format!(r#"{{"email":"{}", "password": "password", "lifetime": "no-expiration" }}"#, email_1),
        StatusCode::OK
    ).await;

}