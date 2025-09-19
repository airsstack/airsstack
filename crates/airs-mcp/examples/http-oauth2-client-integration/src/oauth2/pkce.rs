// PKCE (Proof Key for Code Exchange) implementation

// Internal module imports
use crate::{create_code_challenge, generate_random_string, PkceChallenge};

/// PKCE challenge generator for OAuth2 security
pub struct PkceGenerator;

impl PkceGenerator {
    /// Create a new PKCE generator
    pub fn new() -> Self {
        Self
    }

    /// Generate a PKCE challenge with code verifier and code challenge
    pub fn generate_challenge(&self) -> PkceChallenge {
        // Generate a cryptographically random code verifier
        // RFC 7636 recommends 43-128 characters
        let code_verifier = generate_random_string(128);

        // Create SHA256 hash of the code verifier
        let code_challenge = create_code_challenge(&code_verifier);

        PkceChallenge {
            code_verifier,
            code_challenge,
            code_challenge_method: "S256".to_string(),
        }
    }

    /// Validate that a code verifier matches a code challenge
    pub fn validate_challenge(
        &self,
        code_verifier: &str,
        code_challenge: &str,
        method: &str,
    ) -> bool {
        if method != "S256" {
            return false;
        }

        let expected_challenge = create_code_challenge(code_verifier);
        expected_challenge == code_challenge
    }
}

impl Default for PkceGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkce_challenge_generation() {
        let generator = PkceGenerator::new();
        let challenge = generator.generate_challenge();

        assert_eq!(challenge.code_verifier.len(), 128);
        assert!(!challenge.code_challenge.is_empty());
        assert_eq!(challenge.code_challenge_method, "S256");

        // Verify that the challenge can be validated
        assert!(generator.validate_challenge(
            &challenge.code_verifier,
            &challenge.code_challenge,
            &challenge.code_challenge_method
        ));
    }

    #[test]
    fn test_pkce_challenge_uniqueness() {
        let generator = PkceGenerator::new();
        let challenge1 = generator.generate_challenge();
        let challenge2 = generator.generate_challenge();

        assert_ne!(challenge1.code_verifier, challenge2.code_verifier);
        assert_ne!(challenge1.code_challenge, challenge2.code_challenge);
    }

    #[test]
    fn test_pkce_validation_failure() {
        let generator = PkceGenerator::new();
        let challenge = generator.generate_challenge();

        // Wrong verifier should fail
        assert!(!generator.validate_challenge(
            "wrong-verifier",
            &challenge.code_challenge,
            &challenge.code_challenge_method
        ));

        // Wrong method should fail
        assert!(!generator.validate_challenge(
            &challenge.code_verifier,
            &challenge.code_challenge,
            "plain"
        ));
    }
}
