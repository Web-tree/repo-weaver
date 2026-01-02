use serde::Deserialize;
use wit_bindgen::generate;

generate!({
    world: "provider",
    path: "../../wit",
});

use exports::weaver::plugin::secrets::{Guest, SecretError, SecretRequest};
use weaver::plugin::process::{ExecRequest, exec};

struct Component;

#[derive(Deserialize)]
struct SsmParameter {
    #[serde(rename = "Parameter")]
    parameter: Parameter,
}

#[derive(Deserialize)]
struct Parameter {
    #[serde(rename = "Value")]
    value: String,
}

impl Guest for Component {
    fn get_secret(req: SecretRequest) -> Result<String, SecretError> {
        // Construct args: aws ssm get-parameter --name <key> --with-decryption --output json
        let args = vec![
            "ssm".to_string(),
            "get-parameter".to_string(),
            "--name".to_string(),
            req.key.clone(),
            "--with-decryption".to_string(),
            "--output".to_string(),
            "json".to_string(),
        ];

        let exec_req = ExecRequest {
            program: "aws".to_string(),
            args,
            cwd: None,
            env: vec![], // We assume host passes AWS_PROFILE etc via inherit-env
            inherit_env: true,
            stdin: None,
        };

        let result =
            exec(&exec_req).map_err(|e| SecretError::Internal(format!("Exec failed: {}", e)))?;

        if result.status != 0 {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(SecretError::NotFound(format!("AWS CLI Error: {}", stderr)));
        }

        let stdout = String::from_utf8(result.stdout)
            .map_err(|e| SecretError::Internal(format!("Invalid UTF-8: {}", e)))?;

        let ssm_param: SsmParameter = serde_json::from_str(&stdout)
            .map_err(|e| SecretError::Internal(format!("JSON Parse Error: {}", e)))?;

        Ok(ssm_param.parameter.value)
    }
}

export!(Component);
