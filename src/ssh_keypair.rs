use nullnet_liberror::{Error, ErrorHandler, Location, location};
use rand::{Rng, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use tokio::{fs, process::Command};

const WALLGUARD_SYSTEM_EMAIL: &str = "wallguard-system@nullnet.ai";
// @TODO: Generate Random Filename to avoid collisions
const PRIVATE_KEY_PATH: &str = "/tmp/id_ed25519";
const PUBLIC_KEY_PATH: &str = "/tmp/id_ed25519.pub";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSHKeypair {
    pub public_key: String,
    pub private_key: String,
    pub passphrase: String,
}

impl SSHKeypair {
    pub async fn generate() -> Result<Self, Error> {
        let passphrase = Self::random_passphrase(32);

        let status = Command::new("ssh-keygen")
            .args(&[
                "-t",
                "ed25519",
                "-f",
                PRIVATE_KEY_PATH,
                "-C",
                WALLGUARD_SYSTEM_EMAIL,
                "-N",
                &passphrase,
            ])
            .status()
            .await
            .handle_err(location!())?;

        if !status.success() {
            return Err("Failed to generate SSH keypair with ssh-keygen").handle_err(location!());
        }

        let private_key = fs::read_to_string(PRIVATE_KEY_PATH)
            .await
            .handle_err(location!())?;

        let public_key = fs::read_to_string(PUBLIC_KEY_PATH)
            .await
            .handle_err(location!())?;

        fs::remove_file(PRIVATE_KEY_PATH)
            .await
            .handle_err(location!())?;
        fs::remove_file(PUBLIC_KEY_PATH)
            .await
            .handle_err(location!())?;

        Ok(Self {
            public_key,
            private_key,
            passphrase,
        })
    }

    fn random_passphrase(length: usize) -> String {
        rand::rngs::ThreadRng::default()
            .sample_iter(&Alphanumeric)
            .take(length)
            .map(char::from)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs;

    #[tokio::test]
    async fn test_generate_keypair() {
        let keypair = SSHKeypair::generate().await;

        assert!(keypair.is_ok());

        let keypair = keypair.unwrap();

        assert!(!keypair.public_key.is_empty());
        assert!(!keypair.private_key.is_empty());
        assert!(!keypair.passphrase.is_empty());

        assert!(!fs::metadata(PRIVATE_KEY_PATH).await.is_ok());
        assert!(!fs::metadata(PUBLIC_KEY_PATH).await.is_ok());
    }
}
