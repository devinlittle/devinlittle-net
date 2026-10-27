use base64::prelude::*;

use crate::utils::secrets::SECRETS;

pub fn basic_auth_for_grpc(token: &str) -> anyhow::Result<()> {
    let (boney_username, boney_password) = decode_base64_basic_auth(&SECRETS.bone_to_pick)?;

    if is_base64(token) {
        let (username, password) = decode_base64_basic_auth(token)?;
        if boney_username != username && boney_password != password {
            Err(anyhow::anyhow!("incorrect credentils"))
        } else {
            Ok(())
        }
    } else {
        let (username, password) = token
            .split_once(':')
            .map(|(u, p)| (u.to_string(), p.to_string()))
            .unwrap_or_else(|| (String::new(), String::new()));

        if boney_username != username && boney_password != password {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "creds provided are not what i want as papa"
            ))
        }
    }
}

fn decode_base64_basic_auth(creds: &str) -> anyhow::Result<(String, String)> {
    let creds = BASE64_STANDARD.decode(creds)?;

    let creds = String::from_utf8(creds)?.clone();

    let (username, password) = creds
        .split_once(':')
        .map(|(u, p)| (u.to_string(), p.to_string()))
        .unwrap_or_else(|| (String::new(), String::new()));

    Ok((username, password))
}

fn is_base64(s: &str) -> bool {
    BASE64_STANDARD.decode(s.as_bytes()).is_ok()
}
