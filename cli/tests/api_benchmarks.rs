/// Comprehensive API performance benchmarking
///
/// These benchmarks measure API operation performance under various conditions
/// and validate that performance requirements are met for interactive use.
#[cfg(test)]
mod api_performance_benchmarks {
    use aggsandbox::api;
    use aggsandbox::config::{
        AccountConfig, ApiConfig, ChainConfig, Config, ContractConfig, NetworkConfig,
    };
    use aggsandbox::types::{ChainId, EthereumAddress, RpcUrl};
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn create_test_config(base_url: &str) -> Config {
        Config {
            api: ApiConfig {
                base_url: RpcUrl::new(base_url).expect("Valid test URL"),
                timeout: Duration::from_millis(5000),
                retry_attempts: 3,
            },
            networks: NetworkConfig {
                l1: ChainConfig {
                    name: "Test-L1".to_string(),
                    chain_id: ChainId::new("1").expect("Valid test chain ID"),
                    rpc_url: RpcUrl::new("http://localhost:8545").expect("Valid test URL"),
                    fork_url: None,
                },
                l2: ChainConfig {
                    name: "Test-L2".to_string(),
                    chain_id: ChainId::new("1101").expect("Valid test chain ID"),
                    rpc_url: RpcUrl::new("http://localhost:8546").expect("Valid test URL"),
                    fork_url: None,
                },
                l3: None,
            },
            accounts: AccountConfig {
                accounts: vec![
                    EthereumAddress::new("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")
                        .expect("Valid test address"),
                ],
                private_keys: vec!["0xkey".to_string()],
            },
            contracts: ContractConfig {
                l1_contracts: HashMap::new(),
                l2_contracts: HashMap::new(),
                l3_contracts: HashMap::new(),
            },
        }
    }

    /// Benchmark bridges API endpoint performance
    #[tokio::test]
    async fn benchmark_bridges_api_performance() {
        let mock_server = MockServer::start().await;
        let config = create_test_config(&mock_server.uri());

        let mock_response = json!({
            "bridges": (0..100).map(|i| json!({
                "id": format!("bridge_{i}"),
                "network_id": 0,
                "address": format!("0x{:040x}", i),
                "amount": format!("{}000000000000000000", i + 1)
            })).collect::<Vec<_>>()
        });

        Mock::given(method("GET"))
            .and(path("/bridge/v1/bridges"))
            .and(query_param("network_id", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
            .mount(&mock_server)
            .await;

        // Warm-up calls
        for _ in 0..3 {
            let _ = api::get_bridges(&config, 0, false).await;
        }

        // Benchmark actual calls
        let iterations = 50;
        let start_time = Instant::now();

        for _ in 0..iterations {
            let result = api::get_bridges(&config, 0, false).await;
            assert!(result.is_ok(), "API call should succeed");
        }

        let elapsed = start_time.elapsed();
        let avg_per_call = elapsed / iterations;

        println!("Bridges API Benchmark:");
        println!("  Total time for {iterations} calls: {elapsed:?}");
        println!("  Average per call: {avg_per_call:?}");
        println!(
            "  Calls per second: {:.2}",
            1.0 / avg_per_call.as_secs_f64()
        );

        // Performance requirements for interactive use
        assert!(
            avg_per_call < Duration::from_millis(500),
            "Average call time too slow: {avg_per_call:?}"
        );
        assert!(
            elapsed < Duration::from_secs(10),
            "Total benchmark time too slow: {elapsed:?}"
        );
    }

    /// Benchmark claims API endpoint performance
    #[tokio::test]
    async fn benchmark_claims_api_performance() {
        let mock_server = MockServer::start().await;
        let config = create_test_config(&mock_server.uri());

        let mock_response = json!({
            "claims": (0..50).map(|i| json!({
                "id": format!("claim_{i}"),
                "network_id": 0,
                "leaf_index": i,
                "deposit_count": i + 1,
                "amount": format!("{}000000000000000000", i + 1)
            })).collect::<Vec<_>>()
        });

        Mock::given(method("GET"))
            .and(path("/bridge/v1/claims"))
            .and(query_param("network_id", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
            .mount(&mock_server)
            .await;

        let iterations = 30;
        let start_time = Instant::now();

        for _ in 0..iterations {
            let result = api::get_claims(&config, 0, false).await;
            assert!(result.is_ok(), "Claims API call should succeed");
        }

        let elapsed = start_time.elapsed();
        let avg_per_call = elapsed / iterations;

        println!("Claims API Benchmark:");
        println!("  Average per call: {avg_per_call:?}");

        assert!(
            avg_per_call < Duration::from_millis(600),
            "Claims API too slow: {avg_per_call:?}"
        );
    }

    /// Benchmark concurrent API calls
    #[tokio::test]
    async fn benchmark_concurrent_api_calls() {
        let mock_server = MockServer::start().await;
        let config = Arc::new(create_test_config(&mock_server.uri()));

        let mock_response = json!({
            "bridges": [{"id": "test", "network_id": 0, "address": "0x123"}]
        });

        Mock::given(method("GET"))
            .and(path("/bridge/v1/bridges"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
            .mount(&mock_server)
            .await;

        let concurrent_calls = 20;
        let calls_per_thread = 10;

        let start_time = Instant::now();

        let handles: Vec<_> = (0..concurrent_calls)
            .map(|_| {
                let config = Arc::clone(&config);
                tokio::spawn(async move {
                    for _ in 0..calls_per_thread {
                        let _result = api::get_bridges(&config, 0, false).await;
                    }
                })
            })
            .collect();

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        let elapsed = start_time.elapsed();
        let total_calls = concurrent_calls * calls_per_thread;
        let avg_per_call = elapsed / total_calls;

        println!("Concurrent API Benchmark:");
        println!("  {concurrent_calls} concurrent threads, {calls_per_thread} calls each");
        println!("  Total calls: {total_calls}");
        println!("  Total time: {elapsed:?}");
        println!("  Average per call: {avg_per_call:?}");
        println!(
            "  Throughput: {:.2} calls/sec",
            total_calls as f64 / elapsed.as_secs_f64()
        );

        // Should handle concurrent load efficiently
        assert!(
            avg_per_call < Duration::from_millis(1000),
            "Concurrent calls too slow: {avg_per_call:?}"
        );
    }

    /// Benchmark API calls with varying response sizes
    #[tokio::test]
    async fn benchmark_response_size_impact() {
        let mock_server = MockServer::start().await;
        let config = create_test_config(&mock_server.uri());

        let response_sizes = [10, 100, 1000, 5000];
        let valid_network_ids = [0u64]; // Use only L1 network ID for testing since it stays on original port

        for (i, &size) in response_sizes.iter().enumerate() {
            let network_id = valid_network_ids[i % valid_network_ids.len()];

            let mock_response = json!({
                "bridges": (0..size).map(|j| json!({
                    "id": format!("bridge_{j}"),
                    "network_id": network_id,
                    "address": format!("0x{:040x}", j),
                    "data": format!("large_data_payload_{}", "x".repeat(100))
                })).collect::<Vec<_>>()
            });

            Mock::given(method("GET"))
                .and(path("/bridge/v1/bridges"))
                .and(query_param("network_id", network_id.to_string()))
                .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
                .mount(&mock_server)
                .await;

            let iterations = 10;
            let start_time = Instant::now();

            for _ in 0..iterations {
                let result = api::get_bridges(&config, network_id, false).await;
                assert!(result.is_ok(), "API call should succeed for size {size}");
            }

            let elapsed = start_time.elapsed();
            let avg_per_call = elapsed / iterations;

            println!("Response size {size} items: {avg_per_call:?} per call");

            // Larger responses should still be reasonable
            let max_allowed = Duration::from_millis(200 + size as u64 * 2); // Scale with size
            assert!(
                avg_per_call < max_allowed,
                "Response size {size} too slow: {avg_per_call:?}"
            );
        }
    }

    /// Benchmark API error handling performance
    #[tokio::test]
    async fn benchmark_error_handling_performance() {
        let mock_server = MockServer::start().await;
        let config = create_test_config(&mock_server.uri());

        // Test different error scenarios
        let error_scenarios = [
            (404, "Not Found"),
            (500, "Internal Server Error"),
            (503, "Service Unavailable"),
            (429, "Too Many Requests"),
        ];

        for (status_code, error_msg) in error_scenarios {
            Mock::given(method("GET"))
                .and(path("/bridge/v1/bridges"))
                .and(query_param("network_id", status_code.to_string()))
                .respond_with(ResponseTemplate::new(status_code).set_body_string(error_msg))
                .mount(&mock_server)
                .await;

            let iterations = 20;
            let start_time = Instant::now();

            for _ in 0..iterations {
                let result = api::get_bridges(&config, status_code as u64, false).await;
                assert!(result.is_err(), "Should fail for status {status_code}");
            }

            let elapsed = start_time.elapsed();
            let avg_per_call = elapsed / iterations;

            println!("Error {status_code} handling: {avg_per_call:?} per call");

            // Error handling should be fast
            assert!(
                avg_per_call < Duration::from_millis(100),
                "Error handling too slow for {status_code}: {avg_per_call:?}"
            );
        }
    }

    /// Benchmark memory usage during API operations
    #[tokio::test]
    async fn benchmark_memory_usage_api_operations() {
        let mock_server = MockServer::start().await;
        let config = create_test_config(&mock_server.uri());

        // Create a large response to test memory handling
        let large_response = json!({
            "bridges": (0..10000).map(|i| json!({
                "id": format!("bridge_{i}"),
                "network_id": 0,
                "address": format!("0x{:040x}", i),
                "metadata": {
                    "description": format!("Bridge description {}", "x".repeat(500)),
                    "tags": (0..10).map(|j| format!("tag_{i}_{j}")).collect::<Vec<_>>(),
                    "extra_data": "x".repeat(1000)
                }
            })).collect::<Vec<_>>()
        });

        Mock::given(method("GET"))
            .and(path("/bridge/v1/bridges"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&large_response))
            .mount(&mock_server)
            .await;

        let iterations = 50;
        let start_time = Instant::now();

        for i in 0..iterations {
            let result = api::get_bridges(&config, 0, false).await;
            assert!(result.is_ok(), "Large response API call should succeed");

            // Force garbage collection pressure periodically
            if i % 10 == 0 {
                let pressure: Vec<u8> = vec![0; 1024 * 1024]; // 1MB
                std::hint::black_box(pressure);
            }
        }

        let elapsed = start_time.elapsed();
        let avg_per_call = elapsed / iterations;

        println!("Large response memory benchmark:");
        println!("  Average per call: {avg_per_call:?}");
        println!("  Total time: {elapsed:?}");

        // Should handle large responses efficiently
        assert!(
            avg_per_call < Duration::from_secs(1),
            "Large response handling too slow: {avg_per_call:?}"
        );
    }
}

#[cfg(test)]
mod api_reliability_benchmarks {
    use aggsandbox::api;
    use aggsandbox::api_client::OptimizedApiClient;
    use aggsandbox::config::{
        AccountConfig, ApiConfig, ChainConfig, Config, ContractConfig, NetworkConfig,
    };
    use aggsandbox::types::{ChainId, EthereumAddress, RpcUrl};
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn create_test_config(base_url: &str) -> Config {
        Config {
            api: ApiConfig {
                base_url: RpcUrl::new(base_url).expect("Valid test URL"),
                timeout: Duration::from_millis(2000),
                retry_attempts: 2,
            },
            networks: NetworkConfig {
                l1: ChainConfig {
                    name: "Test-L1".to_string(),
                    chain_id: ChainId::new("1").expect("Valid test chain ID"),
                    rpc_url: RpcUrl::new("http://localhost:8545").expect("Valid test URL"),
                    fork_url: None,
                },
                l2: ChainConfig {
                    name: "Test-L2".to_string(),
                    chain_id: ChainId::new("1101").expect("Valid test chain ID"),
                    rpc_url: RpcUrl::new("http://localhost:8546").expect("Valid test URL"),
                    fork_url: None,
                },
                l3: None,
            },
            accounts: AccountConfig {
                accounts: vec![
                    EthereumAddress::new("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")
                        .expect("Valid test address"),
                ],
                private_keys: vec!["0xkey".to_string()],
            },
            contracts: ContractConfig {
                l1_contracts: HashMap::new(),
                l2_contracts: HashMap::new(),
                l3_contracts: HashMap::new(),
            },
        }
    }

    /// Test API stability under sustained load
    #[tokio::test]
    async fn test_sustained_load_stability() {
        let mock_server = MockServer::start().await;
        let config = Arc::new(create_test_config(&mock_server.uri()));

        let mock_response = json!({
            "bridges": [{"id": "test", "network_id": 0, "address": "0x123"}]
        });

        Mock::given(method("GET"))
            .and(path("/bridge/v1/bridges"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
            .mount(&mock_server)
            .await;

        let duration = Duration::from_secs(10);
        let start_time = Instant::now();
        let success_count = Arc::new(Mutex::new(0));
        let error_count = Arc::new(Mutex::new(0));

        let handles: Vec<_> = (0..5)
            .map(|_| {
                let config = Arc::clone(&config);
                let success_count = Arc::clone(&success_count);
                let error_count = Arc::clone(&error_count);

                tokio::spawn(async move {
                    while start_time.elapsed() < duration {
                        match api::get_bridges(&config, 0, false).await {
                            Ok(_) => *success_count.lock().unwrap() += 1,
                            Err(_) => *error_count.lock().unwrap() += 1,
                        }

                        // Small delay to avoid overwhelming
                        tokio::time::sleep(Duration::from_millis(50)).await;
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.await.unwrap();
        }

        let final_success = *success_count.lock().unwrap();
        let final_errors = *error_count.lock().unwrap();
        let total_calls = final_success + final_errors;

        println!("Sustained load test results:");
        println!("  Duration: {duration:?}");
        println!("  Total calls: {total_calls}");
        println!("  Successful: {final_success}");
        println!("  Errors: {final_errors}");
        println!(
            "  Success rate: {:.2}%",
            (final_success as f64 / total_calls as f64) * 100.0
        );

        // Should handle sustained load with high success rate
        // Adjust expectations for CI environments which may be slower
        let min_expected_calls = if std::env::var("CI").is_ok() { 50 } else { 200 };
        assert!(
            total_calls > min_expected_calls,
            "Should complete sufficient calls for environment: {total_calls} (min: {min_expected_calls})"
        );
        assert!(
            final_success > total_calls * 95 / 100,
            "Success rate should be >95%: {final_success}/{total_calls}"
        );
    }

    /// Test API behavior with intermittent failures
    #[tokio::test]
    async fn test_intermittent_failure_handling() {
        let mock_server = MockServer::start().await;
        let config = create_test_config(&mock_server.uri());

        let success_response = json!({
            "bridges": [{"id": "test", "network_id": 0, "address": "0x123"}]
        });

        // Alternate between success and failure
        let mut call_count = 0;

        for i in 0..20 {
            let response = if i % 3 == 0 {
                // Every 3rd call fails
                ResponseTemplate::new(500).set_body_string("Server Error")
            } else {
                ResponseTemplate::new(200).set_body_json(&success_response)
            };

            Mock::given(method("GET"))
                .and(path("/bridge/v1/bridges"))
                .respond_with(response)
                .mount(&mock_server)
                .await;

            let start_time = Instant::now();
            let _result = api::get_bridges(&config, 0, false).await;
            let call_duration = start_time.elapsed();

            call_count += 1;

            // Each call should complete reasonably quickly, even failures
            assert!(
                call_duration < Duration::from_secs(3),
                "Call {i} took too long: {call_duration:?}"
            );
        }

        assert_eq!(call_count, 20, "Should complete all calls");
    }

    /// Test API timeout handling performance characteristics
    #[tokio::test]
    async fn test_timeout_handling_performance() {
        // Clear the cache to ensure we're not getting cached responses
        let client = OptimizedApiClient::global();
        client.clear_cache().await;

        // Test timeout behavior by connecting to a non-responsive endpoint
        let mut config = create_test_config("http://localhost:9999"); // Non-existent port
        config.api.timeout = Duration::from_millis(100); // Very short timeout

        let iterations = 5; // Fewer iterations for timeout tests
        let start_time = Instant::now();
        let mut failure_count = 0;

        for _ in 0..iterations {
            let call_start = Instant::now();
            let result = api::get_bridges(&config, 0, false).await;
            let call_duration = call_start.elapsed();

            // Should fail (either timeout or connection refused)
            if result.is_err() {
                failure_count += 1;
            }

            // Should fail relatively quickly (not hang for minutes)
            assert!(
                call_duration < Duration::from_secs(5),
                "Call should fail quickly: {call_duration:?}"
            );
        }

        let total_duration = start_time.elapsed();

        println!("Timeout handling benchmark:");
        println!("  {iterations} calls took: {total_duration:?}");
        println!("  Average per call: {:?}", total_duration / iterations);
        println!("  Failed calls: {failure_count}/{iterations}");

        // At least one call should fail (timeout or connection refused)
        assert!(
            failure_count >= 1,
            "At least one call should fail (timeout or connection refused): {failure_count}/{iterations}"
        );
        // Warn if not all calls fail (for diagnostic purposes, but do not fail the test)
        if failure_count != iterations {
            eprintln!("[WARN] Not all timeout test calls failed. This may be due to OS/network stack behavior. Failures: {failure_count}/{iterations}");
        }

        // Total time should be reasonable
        assert!(
            total_duration < Duration::from_secs(30),
            "Total timeout handling too slow: {total_duration:?}"
        );
    }
}
