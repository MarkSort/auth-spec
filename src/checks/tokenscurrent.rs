use hyper::StatusCode;

pub async fn check(c: &mut crate::checker::Checker) {
    // error cases
    c.path = "/tokens/current";
    c.post_no_body("method not allowed", StatusCode::METHOD_NOT_ALLOWED).await;

    // success cases
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
    let mut token_1 = None;
    if let Some(json_response) = json_response {
        if let Some(id) = c.get_property_string(&json_response, "id") {
            if let Some(secret) = c.get_property_string(&json_response, "secret") {
                if let Some(lifetime) = c.get_property_string(&json_response, "lifetime") {
                    if let Some(created) = c.get_property_i64(&json_response, "created") {
                        if let Some(last_active) = c.get_property_i64(&json_response, "last_active") {
                            token_1 = Some(Token{ id, secret, lifetime, created, last_active });
                        }
                    }
                }
            }
        }
    } else {
        c.fail("token create response was not json".into());
    }

    c.path = "/tokens/current";
    if let Some(token_1) = token_1 {
        let (json_response, _) = c.get_with_token(
            "correct response format; check",
            token_1.secret.clone(),
            StatusCode::OK,
        ).await;
        if let Some(json_response) = json_response {
            if let Some(id) = c.get_property_string(&json_response, "id") {
                c.check(
                    token_1.id == id,
                    "property 'id' does not match create response".into()
                );
            }
            if let Some(lifetime) = c.get_property_string(&json_response, "lifetime") {
                c.check(
                    token_1.lifetime == lifetime,
                    "property 'lifetime' does not match create response".into()
                );
            }
            if let Some(created) = c.get_property_i64(&json_response, "created") {
                c.check(
                    token_1.created == created,
                    "property 'created' does not match create response".into()
                );
            }
            if let Some(last_active) = c.get_property_i64(&json_response, "last_active") {
                c.check(
                    token_1.last_active == last_active,
                    "property 'last_active' does not match create response".into()
                );
            }
        } else {
            c.fail("response was not json".into());
        }

        let (json_response, _) = c.delete_with_token(
            "correct response format; check",
            token_1.secret,
            StatusCode::OK,
        ).await;
        if let Some(json_response) = json_response {
            if c.get_property_string(&json_response, "success").is_some() {
                c.pass(1);
            } else {
                c.fail("response did not include success property".into());
            }
        } else {
            c.fail("response was not json".into());
        }

    }
}

struct Token {
    id: String,
    secret: String,
    lifetime: String,
    created: i64,
    last_active: i64,
}
