use hyper::StatusCode;

pub async fn check(c: &mut crate::checker::Checker) {
    let email_1 = format!("test+{:0>8x}@example.com", rand::random::<u32>());

    c.path = "/users";
    c.post(
        "invalid credentials; wrong password; create",
        format!(r#"{{ "email": "{}", "password": "password" }}"#, email_1),
        StatusCode::OK,
    ).await;
    c.path = "/tokens";
    let (json_response, _) = c.post(
        "correct response format; check",
        format!(r#"{{ "email": "{}", "password": "password", "lifetime": "no-expiration" }}"#, email_1),
        StatusCode::OK,
    ).await;
    let mut token_secret = None;
    if let Some(json_response) = json_response {
        token_secret = c.get_property_string(&json_response, "secret");
    } else {
        c.fail("token create response was not json".into());
    }

    if let Some(token_secret) = token_secret {
        // error cases
        c.path = "/tokens/current/refresh";
        c.get_with_token("method not allowed", token_secret.clone(), StatusCode::METHOD_NOT_ALLOWED).await;

        // success cases
        c.path = "/tokens/current/refresh";
        let (json_response, _) = c.post_with_token(
            "correct response format; check",
            token_secret,
            StatusCode::OK,
        ).await;
        if let Some(json_response) = json_response {
            c.get_property_string(&json_response, "id");
            c.get_property_string(&json_response, "lifetime");
            c.get_property_i64(&json_response, "created");
            c.get_property_i64(&json_response, "last_active");
        } else {
            c.fail("response was not json".into());
        }
    }
}
