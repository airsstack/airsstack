//! Command line argument parsing for OAuth2 client integration

// Layer 2: Third-party crate imports
use clap::{Arg, ArgMatches, Command};

/// Parse command line arguments and return matches
pub fn parse_args() -> ArgMatches {
    Command::new("http-oauth2-client")
        .version("0.1.0")
        .author("AIRS Stack Contributors")
        .about("HTTP MCP client with OAuth2 authentication")
        .arg(
            Arg::new("auth-server")
                .long("auth-server")
                .value_name("URL")
                .help("OAuth2 authorization server URL")
                .default_value("http://localhost:8080"),
        )
        .arg(
            Arg::new("mcp-server")
                .long("mcp-server")
                .value_name("URL")
                .help("OAuth2-protected MCP server URL")
                .default_value("http://localhost:8081"),
        )
        .arg(
            Arg::new("client-id")
                .long("client-id")
                .value_name("ID")
                .help("OAuth2 client ID")
                .default_value("test-client"),
        )
        .arg(
            Arg::new("scope")
                .long("scope")
                .value_name("SCOPE")
                .help("OAuth2 scope to request")
                .default_value("mcp:read mcp:write"),
        )
        .arg(
            Arg::new("interactive")
                .long("interactive")
                .help("Run in interactive mode with user consent simulation")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches()
}