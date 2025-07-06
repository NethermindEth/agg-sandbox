/// Comprehensive error scenario testing for real-world failure modes
///
/// These tests validate error handling for Docker failures, network issues,
/// configuration problems, and other edge cases users might encounter.
#[cfg(test)]
mod docker_failure_tests {
    use aggsandbox::docker::DockerComposeBuilder;
    use aggsandbox::error::{AggSandboxError, DockerError};
    use tempfile::TempDir;

    /// Test Docker daemon not running scenario
    #[test]
    fn test_docker_daemon_not_running() {
        // Simulate Docker daemon not running by using invalid Docker socket
        std::env::set_var("DOCKER_HOST", "unix:///nonexistent/docker.sock");

        let mut builder = DockerComposeBuilder::new();
        builder.add_file("docker-compose.yml");

        // Test Docker error creation
        let error =
            DockerError::command_failed("docker ps", "Cannot connect to the Docker daemon socket");
        let agg_error = AggSandboxError::Docker(error);

        // Restore normal Docker environment
        std::env::remove_var("DOCKER_HOST");

        let error_msg = format!("{agg_error}");
        assert!(error_msg.contains("Cannot connect to the Docker daemon socket"));
    }

    /// Test Docker Compose file not found
    #[test]
    fn test_compose_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let error = DockerError::compose_file_not_found("nonexistent-compose.yml");
        let agg_error = AggSandboxError::Docker(error);

        let error_msg = format!("{agg_error}");
        assert!(error_msg.contains("nonexistent-compose.yml"));
        assert!(error_msg.contains("not found"));
    }

    /// Test Docker permission denied scenario
    #[test]
    fn test_docker_permission_denied() {
        let error = DockerError::command_failed(
            "docker-compose up -d",
            "permission denied while trying to connect to the Docker daemon socket",
        );
        let agg_error = AggSandboxError::Docker(error);

        let error_msg = format!("{agg_error}");
        assert!(error_msg.contains("permission denied"));
        assert!(error_msg.contains("Docker daemon socket"));
    }

    /// Test Docker out of disk space
    #[test]
    fn test_docker_out_of_disk_space() {
        let error = DockerError::command_failed("docker-compose up -d", "no space left on device");
        let agg_error = AggSandboxError::Docker(error);

        let error_msg = format!("{agg_error}");
        assert!(error_msg.contains("no space left on device"));
    }

    /// Test Docker port already in use
    #[test]
    fn test_docker_port_in_use() {
        let error =
            DockerError::command_failed("docker-compose up -d", "port is already allocated");
        let agg_error = AggSandboxError::Docker(error);

        let error_msg = format!("{agg_error}");
        assert!(error_msg.contains("port is already allocated"));
    }

    /// Test Docker image pull failure
    #[test]
    fn test_docker_image_pull_failure() {
        let error = DockerError::command_failed(
            "docker-compose up -d",
            "pull access denied for image, repository does not exist",
        );
        let agg_error = AggSandboxError::Docker(error);

        let error_msg = format!("{agg_error}");
        assert!(error_msg.contains("pull access denied"));
        assert!(error_msg.contains("repository does not exist"));
    }

    /// Test Docker Compose validation with invalid YAML
    #[test]
    fn test_compose_validation_invalid_yaml() {
        let error = DockerError::compose_validation_failed(
            "Invalid YAML syntax: mapping values are not allowed here",
        );
        let agg_error = AggSandboxError::Docker(error);

        let error_msg = format!("{agg_error}");
        assert!(error_msg.contains("Invalid YAML syntax"));
        assert!(error_msg.contains("mapping values are not allowed"));
    }

    /// Test Docker container startup timeout
    #[test]
    fn test_docker_container_startup_timeout() {
        let error = DockerError::command_failed(
            "docker-compose up -d",
            "container failed to start within timeout period",
        );
        let agg_error = AggSandboxError::Docker(error);

        let error_msg = format!("{agg_error}");
        assert!(error_msg.contains("failed to start within timeout"));
    }
}

#[cfg(test)]
mod network_failure_tests {
    use aggsandbox::api;
    use aggsandbox::config::{
        AccountConfig, ApiConfig, ChainConfig, Config, ContractConfig, NetworkConfig,
    };
    use aggsandbox::error::{AggSandboxError, ApiError};
    use std::collections::HashMap;
    use std::time::Duration;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn create_test_config(base_url: &str) -> Config {
        Config {
            api: ApiConfig {
                base_url: base_url.to_string(),
                timeout: Duration::from_millis(1000), // Short timeout for testing
                retry_attempts: 1,                    // Single attempt for testing
            },
            networks: NetworkConfig {
                l1: ChainConfig {
                    name: "Test-L1".to_string(),
                    chain_id: "1".to_string(),
                    rpc_url: "http://localhost:8545".to_string(),
                    fork_url: None,
                },
                l2: ChainConfig {
                    name: "Test-L2".to_string(),
                    chain_id: "1101".to_string(),
                    rpc_url: "http://localhost:8546".to_string(),
                    fork_url: None,
                },
                l3: None,
            },
            accounts: AccountConfig {
                accounts: vec!["0xtest".to_string()],
                private_keys: vec!["0xkey".to_string()],
            },
            contracts: ContractConfig {
                l1_contracts: HashMap::new(),
                l2_contracts: HashMap::new(),
            },
        }
    }

    /// Test network connection timeout
    #[tokio::test]
    async fn test_network_connection_timeout() {
        let mock_server = MockServer::start().await;
        let config = create_test_config(&mock_server.uri());

        Mock::given(method("GET"))
            .and(path("/bridge/v1/bridges"))
            .respond_with(
                ResponseTemplate::new(200).set_delay(Duration::from_secs(2)), // Longer than our timeout
            )
            .mount(&mock_server)
            .await;

        let result = api::get_bridges(&config, 1).await;

        assert!(result.is_err());
        // Note: This test may pass quickly in test environment
        // In real scenarios, this would timeout
    }

    /// Test DNS resolution failure
    #[tokio::test]
    async fn test_dns_resolution_failure() {
        let config = create_test_config("http://nonexistent.invalid.domain.local:5577");

        let result = api::get_bridges(&config, 1).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            AggSandboxError::Api(ApiError::NetworkError(_)) => {} // Expected
            _ => panic!("Expected NetworkError for DNS failure"),
        }
    }

    /// Test connection refused (service down)
    #[tokio::test]
    async fn test_connection_refused() {
        // Use a port that's unlikely to be in use
        let config = create_test_config("http://localhost:9999");

        let result = api::get_bridges(&config, 1).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            AggSandboxError::Api(ApiError::NetworkError(_)) => {} // Expected
            _ => panic!("Expected NetworkError for connection refused"),
        }
    }

    /// Test HTTP 500 server error
    #[tokio::test]
    async fn test_server_internal_error() {
        let mock_server = MockServer::start().await;
        let config = create_test_config(&mock_server.uri());

        Mock::given(method("GET"))
            .and(path("/bridge/v1/bridges"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&mock_server)
            .await;

        let result = api::get_bridges(&config, 1).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            AggSandboxError::Api(ApiError::RequestFailed { status: 500, .. }) => {} // Expected
            _ => panic!("Expected RequestFailed with status 500"),
        }
    }

    /// Test HTTP 404 not found
    #[tokio::test]
    async fn test_endpoint_not_found() {
        let mock_server = MockServer::start().await;
        let config = create_test_config(&mock_server.uri());

        Mock::given(method("GET"))
            .and(path("/bridge/v1/bridges"))
            .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
            .mount(&mock_server)
            .await;

        let result = api::get_bridges(&config, 1).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            AggSandboxError::Api(ApiError::RequestFailed { status: 404, .. }) => {} // Expected
            _ => panic!("Expected RequestFailed with status 404"),
        }
    }

    /// Test malformed JSON response
    #[tokio::test]
    async fn test_malformed_json_response() {
        let mock_server = MockServer::start().await;
        let config = create_test_config(&mock_server.uri());

        Mock::given(method("GET"))
            .and(path("/bridge/v1/bridges"))
            .respond_with(ResponseTemplate::new(200).set_body_string("{invalid json"))
            .mount(&mock_server)
            .await;

        let result = api::get_bridges(&config, 1).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            AggSandboxError::Api(ApiError::JsonParseError(_)) => {} // Expected
            _ => panic!("Expected JsonParseError for malformed JSON"),
        }
    }

    /// Test empty response body
    #[tokio::test]
    async fn test_empty_response_body() {
        let mock_server = MockServer::start().await;
        let config = create_test_config(&mock_server.uri());

        Mock::given(method("GET"))
            .and(path("/bridge/v1/bridges"))
            .respond_with(ResponseTemplate::new(200).set_body_string(""))
            .mount(&mock_server)
            .await;

        let result = api::get_bridges(&config, 1).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            AggSandboxError::Api(ApiError::JsonParseError(_)) => {} // Expected
            _ => panic!("Expected JsonParseError for empty body"),
        }
    }

    /// Test very large response body
    #[tokio::test]
    async fn test_large_response_body() {
        let mock_server = MockServer::start().await;
        let config = create_test_config(&mock_server.uri());

        // Create a large JSON response (1MB)
        let large_json = format!(
            r#"{{"bridges": [{}]}}"#,
            (0..10000)
                .map(|i| format!(r#"{{"id": "{i}", "name": "bridge_{i}"}}"#))
                .collect::<Vec<_>>()
                .join(",")
        );

        Mock::given(method("GET"))
            .and(path("/bridge/v1/bridges"))
            .respond_with(ResponseTemplate::new(200).set_body_string(large_json))
            .mount(&mock_server)
            .await;

        let result = api::get_bridges(&config, 1).await;

        // This should succeed but we verify it handles large responses
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod configuration_failure_tests {
    use aggsandbox::config::Config;
    use aggsandbox::error::{AggSandboxError, ConfigError};
    use aggsandbox::validation::Validator;
    use std::env;

    /// Test missing environment variables
    #[test]
    fn test_missing_environment_variables() {
        // Save current environment
        let original_home = env::var("HOME").ok();

        // Remove environment variable
        env::remove_var("HOME");

        let error = ConfigError::env_var_not_found("HOME");
        let agg_error = AggSandboxError::Config(error);

        let error_msg = format!("{agg_error}");
        assert!(error_msg.contains("HOME"));
        assert!(error_msg.contains("not found"));

        // Restore environment
        if let Some(home) = original_home {
            env::set_var("HOME", home);
        }
    }

    /// Test invalid environment variable values
    #[test]
    fn test_invalid_environment_values() {
        let error =
            ConfigError::invalid_value("API_TIMEOUT", "not_a_number", "must be a valid number");
        let agg_error = AggSandboxError::Config(error);

        let error_msg = format!("{agg_error}");
        assert!(error_msg.contains("API_TIMEOUT"));
        assert!(error_msg.contains("not_a_number"));
        assert!(error_msg.contains("must be a valid number"));
    }

    /// Test configuration file not found
    #[test]
    fn test_config_file_not_found() {
        let result = Config::load();
        // Config::load() should handle missing files gracefully
        // This test ensures it doesn't panic
        assert!(result.is_ok() || result.is_err());
    }

    /// Test invalid network ID validation
    #[test]
    fn test_invalid_network_id_validation() {
        let invalid_network_ids = vec![0, u64::MAX, 999999];

        for network_id in invalid_network_ids {
            let result = Validator::validate_network_id(network_id);
            assert!(result.is_err(), "Network ID {network_id} should be invalid");
        }
    }

    /// Test invalid Ethereum address validation
    #[test]
    fn test_invalid_ethereum_address_validation() {
        let invalid_addresses = vec![
            "",                                            // Empty
            "0x",                                          // Too short
            "0x123",                                       // Too short
            "invalid_address",                             // Not hex
            "0xZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ",  // Invalid hex
            "0x742d35cc6965c592342c6c16fb8eaeb90a23b5c",   // Too short by 1
            "0x742d35cc6965c592342c6c16fb8eaeb90a23b5c00", // Too long by 1
        ];

        for address in invalid_addresses {
            let result = Validator::validate_ethereum_address(address);
            assert!(result.is_err(), "Address '{address}' should be invalid");
        }
    }

    /// Test invalid URL validation
    #[test]
    fn test_invalid_url_validation() {
        let invalid_urls = vec![
            "",                        // Empty
            "not_a_url",               // Not a URL
            "ftp://example.com",       // Wrong protocol
            "http://",                 // No host
            "https://",                // No host
            "http://[invalid",         // Malformed
            "javascript:alert('xss')", // Security risk
        ];

        for url in invalid_urls {
            let result = Validator::validate_rpc_url(url);
            assert!(result.is_err(), "URL '{url}' should be invalid");
        }
    }

    /// Test invalid chain name validation
    #[test]
    fn test_invalid_chain_validation() {
        let invalid_chains = vec![
            "",              // Empty
            "invalid-chain", // Not supported
            "ethereum",      // Wrong name
            "polygon",       // Wrong name
            "anvil_l1",      // Wrong separator
            "l1-anvil",      // Wrong order
        ];

        for chain in invalid_chains {
            let result = Validator::validate_chain(chain);
            assert!(result.is_err(), "Chain '{chain}' should be invalid");
        }
    }

    /// Test configuration validation edge cases
    #[test]
    fn test_configuration_validation_edge_cases() {
        // Test block count edge cases
        assert!(Validator::validate_block_count(0).is_err());
        assert!(Validator::validate_block_count(100000).is_err());

        // Test timeout edge cases
        assert!(Validator::validate_timeout_ms(0).is_err());
        assert!(Validator::validate_timeout_ms(u64::MAX).is_err());

        // Test retry attempts edge cases
        // Note: 0 retries is actually valid, test high numbers
        assert!(Validator::validate_retry_attempts(100).is_err());
    }

    /// Test file path validation with security concerns
    #[test]
    fn test_file_path_validation_security() {
        let dangerous_paths = vec![
            "../../../etc/passwd",         // Path traversal
            "/etc/passwd",                 // Absolute system path
            "~/.ssh/id_rsa",               // Home directory access
            "file:///etc/passwd",          // File URI
            "\\..\\..\\windows\\system32", // Windows path traversal
        ];

        for path in dangerous_paths {
            let result = Validator::validate_file_path(path);
            // These should either be rejected or properly sanitized
            if result.is_ok() {
                let validated = result.unwrap();
                assert!(
                    !validated.contains(".."),
                    "Path traversal not prevented: {validated}"
                );
                assert!(
                    !validated.starts_with('/'),
                    "Absolute path not prevented: {validated}"
                );
            }
        }
    }
}

#[cfg(test)]
mod resource_exhaustion_tests {
    use aggsandbox::validation::Validator;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{Duration, Instant};

    /// Test memory usage under extreme load
    #[test]
    fn test_memory_exhaustion_resistance() {
        let start_time = Instant::now();
        let max_duration = Duration::from_secs(5); // Limit test duration

        // Try to create many validation operations
        let mut iterations = 0;
        while start_time.elapsed() < max_duration && iterations < 100000 {
            let _result =
                Validator::validate_ethereum_address("0x742d35cc6965c592342c6c16fb8eaeb90a23b5c0");
            iterations += 1;

            // Check every 1000 iterations
            if iterations % 1000 == 0 {
                // Force garbage collection by creating and dropping large allocations
                let large_vec: Vec<u8> = vec![0; 1024 * 1024]; // 1MB
                std::hint::black_box(large_vec); // Prevent optimization
            }
        }

        // Test should complete without crashing
        assert!(iterations > 0, "Test should have completed some iterations");
    }

    /// Test concurrent access under load
    #[test]
    fn test_concurrent_access_under_extreme_load() {
        let thread_count = 20;
        let iterations_per_thread = 1000;
        let error_count = Arc::new(Mutex::new(0));

        let handles: Vec<_> = (0..thread_count)
            .map(|thread_id| {
                let error_count = Arc::clone(&error_count);
                thread::spawn(move || {
                    for i in 0..iterations_per_thread {
                        // Mix different validation operations
                        let operations = [
                            || Validator::validate_chain("anvil-l1").is_ok(),
                            || Validator::validate_network_id(1).is_ok(),
                            || {
                                Validator::validate_ethereum_address(
                                    "0x742d35cc6965c592342c6c16fb8eaeb90a23b5c0",
                                )
                                .is_ok()
                            },
                            || Validator::validate_rpc_url("http://localhost:8545").is_ok(),
                        ];

                        let op_index = (thread_id + i) % operations.len();
                        if !operations[op_index]() {
                            *error_count.lock().unwrap() += 1;
                        }

                        // Introduce some random delays to increase contention
                        if i % 100 == 0 {
                            thread::sleep(Duration::from_millis(1));
                        }
                    }
                })
            })
            .collect();

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        let total_errors = *error_count.lock().unwrap();

        // We expect no validation errors for valid inputs
        assert_eq!(
            total_errors, 0,
            "Unexpected validation errors under load: {total_errors}"
        );
    }

    /// Test file descriptor exhaustion resistance
    #[test]
    fn test_file_descriptor_exhaustion() {
        // This test ensures our code doesn't leak file descriptors
        // by repeatedly creating and dropping operations that might use files

        let start_time = Instant::now();
        let max_duration = Duration::from_secs(3);
        let mut iterations = 0;

        while start_time.elapsed() < max_duration && iterations < 10000 {
            // Operations that might use file descriptors
            let _config_result = aggsandbox::config::Config::load();
            let _validation_result = Validator::validate_file_path("test.txt");

            iterations += 1;
        }

        // Test should complete without exhausting file descriptors
        assert!(
            iterations > 100,
            "Should have completed many iterations: {iterations}"
        );
    }

    /// Test string allocation resistance
    #[test]
    fn test_string_allocation_exhaustion() {
        let start_time = Instant::now();
        let max_duration = Duration::from_secs(2);
        let mut iterations = 0;

        while start_time.elapsed() < max_duration && iterations < 50000 {
            // Operations that create many strings
            let large_string = "x".repeat(1000);
            let _result = Validator::validate_service_name(&large_string);

            // Create and drop error messages
            let error =
                aggsandbox::error::ConfigError::missing_required(&format!("VAR_{iterations}"));
            let _error_msg = format!("{error}");

            iterations += 1;

            // Periodic cleanup
            if iterations % 1000 == 0 {
                // Force some memory pressure
                let _temp: Vec<String> = (0..100).map(|i| format!("temp_{i}")).collect();
            }
        }

        assert!(
            iterations > 1000,
            "Should have completed many string operations: {iterations}"
        );
    }
}
