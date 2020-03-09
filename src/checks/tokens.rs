use hyper::StatusCode;

pub async fn check(c: &mut crate::checker::Checker) {
    c.path = "/tokens";

    // error cases
    let response = c.post_no_body("no body/content-length", StatusCode::BAD_REQUEST).await;
    c.check_error_response_multi(response, vec!["body", "content-length"]);

    let response = c.post_bad_content_type(
        "content-type other than null or application/json",
        StatusCode::UNSUPPORTED_MEDIA_TYPE,
    ).await;
    c.check_error_response_multi(response, vec!["content-type", "unsupported media type"]);

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
    c.check_error_response(response, "string");

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
    let mut token_1 = None;
    if let Some(json_response) = json_response {
        if let Some(id) = c.get_property_string(&json_response, "id") {
            if let Some(secret) = c.get_property_string(&json_response, "secret") {
                token_1 = Some(Token{ id, secret });
            }
        } else {
            c.get_property_string(&json_response, "secret");
        }
        c.get_property_string(&json_response, "lifetime");
        c.get_property_i64(&json_response, "created");
        c.get_property_i64(&json_response, "last_active");
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

    if let Some(token_1) = token_1 {
        let (json_response, _) = c.get_with_token("correct response format", token_1.secret, StatusCode::OK).await;
        if let Some(json_response) = json_response {

            if let Some(property_value) = json_response.get("tokens") {
                if let Some(tokens) = property_value.as_array() {
                    c.check(tokens.len() == 7, format!("incorrect number of tokens returned: {}", tokens.len()));

                    let mut token_1_found = false;
                    for token in tokens {
                        if let Some(token_id) = c.get_property_string(token, "id") {
                            if token_1.id == token_id {
                                token_1_found = true;
                            }
                        }
                        c.get_property_string(token, "lifetime");
                        c.get_property_i64(token, "created");
                        c.get_property_i64(token, "last_active");
                    }
                    c.check(token_1_found, "current token not found in list of tokens".into());
                } else {
                    c.fail(format!("json '{}' property is not an array", "tokens"));
                }
            } else {
                c.fail(format!("json does not have a '{}' property: {:?}", "tokens", json_response));
            }

        } else {
            c.fail("response is not json".into());
        }
    }
}

struct Token {
    id: String,
    secret: String
}
