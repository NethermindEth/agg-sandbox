/// Performance and stress tests for the AggLayer sandbox CLI
///
/// These tests verify performance characteristics and resource usage
/// to ensure the CLI remains responsive under various conditions.
#[cfg(test)]
mod perf_tests {
    use aggsandbox::config::Config;
    use aggsandbox::error::ConfigError;
    use aggsandbox::validation::Validator;
    use std::time::{Duration, Instant};
    use test_case::test_case;

    #[test]
    fn test_config_loading_performance() {
        // Test that configuration loading is fast enough
        let start = Instant::now();

        for _ in 0..100 {
            let _ = Config::load();
        }

        let elapsed = start.elapsed();

        // Configuration loading should complete well under 1 second for 100 iterations
        assert!(
            elapsed < Duration::from_millis(1000),
            "Config loading too slow: {elapsed:?}"
        );

        // Average per-load should be under 10ms
        let avg_per_load = elapsed / 100;
        assert!(
            avg_per_load < Duration::from_millis(10),
            "Average config load time too slow: {avg_per_load:?}"
        );
    }

    #[test_case("anvil-l1"; "chain l1")]
    #[test_case("anvil-l2"; "chain l2")]
    #[test_case("anvil-l3"; "chain l3")]
    fn test_validation_performance(chain: &str) {
        // Test that validation functions are fast enough for interactive use
        let start = Instant::now();

        for _ in 0..1000 {
            let _ = Validator::validate_chain(chain);
        }

        let elapsed = start.elapsed();

        // 1000 validations should complete in under 100ms
        assert!(
            elapsed < Duration::from_millis(100),
            "Chain validation too slow: {elapsed:?}"
        );
    }

    #[test]
    fn test_address_validation_performance() {
        let valid_address = "0x742d35cc6965c592342c6c16fb8eaeb90a23b5c0";
        let start = Instant::now();

        for _ in 0..1000 {
            let _ = Validator::validate_ethereum_address(valid_address);
        }

        let elapsed = start.elapsed();

        // Address validation should be very fast
        let max_expected_duration = if std::env::var("CI").is_ok() {
            Duration::from_millis(150)
        } else {
            Duration::from_millis(50)
        };
        assert!(
            elapsed < max_expected_duration,
            "Address validation too slow: {elapsed:?} (max: {max_expected_duration:?})"
        );
    }

    #[test]
    fn test_network_id_validation_performance() {
        let start = Instant::now();

        for network_id in [1u64, 1101, 1102, 31337, 31338, 31339] {
            for _ in 0..100 {
                let _ = Validator::validate_network_id(network_id);
            }
        }

        let elapsed = start.elapsed();

        // 600 network ID validations should complete very quickly
        let max_expected_duration = if std::env::var("CI").is_ok() {
            Duration::from_millis(50)
        } else {
            Duration::from_millis(10)
        };
        assert!(
            elapsed < max_expected_duration,
            "Network ID validation too slow: {elapsed:?} (max: {max_expected_duration:?})"
        );
    }

    #[test]
    fn test_batch_validation_performance() {
        let network_ids: Vec<u64> = vec![1, 1101, 1102, 31337, 31338, 31339];
        let start = Instant::now();

        for _ in 0..100 {
            let _result = Validator::validate_batch(network_ids.clone(), |&id| {
                Validator::validate_network_id(id)
            });
        }

        let elapsed = start.elapsed();

        // Batch validation should be efficient
        assert!(
            elapsed < Duration::from_millis(100),
            "Batch validation too slow: {elapsed:?}"
        );
    }

    #[test]
    fn test_error_creation_performance() {
        use aggsandbox::error::{ApiError, ConfigError, DockerError, EventError};

        let start = Instant::now();

        for _ in 0..1000 {
            // Test various error creation patterns
            let _config_err = ConfigError::missing_required("TEST_VAR");
            let _api_err = ApiError::network_error("Test error");
            let _event_err = EventError::invalid_chain("test-chain");
            let _docker_err = DockerError::command_failed("test-cmd", "Test failure");
        }

        let elapsed = start.elapsed();

        // Error creation should be fast
        let max_expected_duration = if std::env::var("CI").is_ok() {
            Duration::from_millis(150)
        } else {
            Duration::from_millis(50)
        };
        assert!(
            elapsed < max_expected_duration,
            "Error creation too slow: {elapsed:?} (max: {max_expected_duration:?})"
        );
    }

    #[test]
    fn test_url_validation_performance() {
        let test_urls = vec![
            "http://localhost:8545",
            "https://rpc.example.com",
            "http://192.168.1.100:8546",
            "https://mainnet.infura.io/v3/key",
        ];

        let start = Instant::now();

        for _ in 0..100 {
            for url in &test_urls {
                let _ = Validator::validate_rpc_url(url);
            }
        }

        let elapsed = start.elapsed();

        // URL validation should be reasonably fast
        assert!(
            elapsed < Duration::from_millis(200),
            "URL validation too slow: {elapsed:?}"
        );
    }

    #[test]
    fn test_service_name_validation_performance() {
        let service_names = vec![
            "anvil-l1",
            "anvil-l2",
            "postgres",
            "redis",
            "zkevm-node",
            "zkevm-prover",
        ];

        let start = Instant::now();

        for _ in 0..200 {
            for name in &service_names {
                let _ = Validator::validate_service_name(name);
            }
        }

        let elapsed = start.elapsed();

        // Service name validation should be reasonably fast (regex compilation can be slow)
        // Adjust expectations for CI environments which may be slower
        let max_expected_duration = if std::env::var("CI").is_ok() {
            Duration::from_millis(500)
        } else {
            Duration::from_millis(200)
        };
        assert!(
            elapsed < max_expected_duration,
            "Service name validation too slow: {elapsed:?} (max: {max_expected_duration:?})"
        );
    }

    #[test]
    fn test_concurrent_validation_performance() {
        use std::sync::Arc;
        use std::thread;

        let addresses = Arc::new(vec![
            "0x742d35cc6965c592342c6c16fb8eaeb90a23b5c0",
            "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
            "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC",
            "0x90F79bf6EB2c4f870365E785982E1f101E93b906",
        ]);

        let start = Instant::now();

        let handles: Vec<_> = (0..4)
            .map(|_| {
                let addresses = Arc::clone(&addresses);
                thread::spawn(move || {
                    for _ in 0..250 {
                        for addr in addresses.iter() {
                            let _ = Validator::validate_ethereum_address(addr);
                        }
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let elapsed = start.elapsed();

        // Concurrent validation should complete in reasonable time
        // 4 threads * 250 iterations * 4 addresses = 4000 total validations
        assert!(
            elapsed < Duration::from_millis(500),
            "Concurrent validation too slow: {elapsed:?}"
        );
    }

    #[test]
    fn test_memory_usage_stability() {
        // Test that repeated operations don't cause memory leaks
        // This is a basic test - in production you'd use tools like valgrind or heaptrack

        let start = Instant::now();

        // Perform many operations that could potentially leak memory
        for i in 0..1000 {
            // Create and drop many validation results
            let chain_result = Validator::validate_chain("anvil-l1");
            let addr_result =
                Validator::validate_ethereum_address("0x742d35cc6965c592342c6c16fb8eaeb90a23b5c0");
            let network_result = Validator::validate_network_id(1);

            // Use the results to prevent optimization
            let _combined = (
                chain_result.is_ok(),
                addr_result.is_ok(),
                network_result.is_ok(),
            );

            // Create and drop error instances
            if i % 100 == 0 {
                let _err = ConfigError::missing_required(&format!("VAR_{i}"));
            }
        }

        let elapsed = start.elapsed();

        // Operations should complete in reasonable time without excessive memory allocation
        assert!(
            elapsed < Duration::from_millis(200),
            "Memory stability test too slow: {elapsed:?}"
        );
    }

    #[test]
    fn test_large_input_handling() {
        // Test that validation functions handle large inputs gracefully

        // Test very long service name (should pass validation but be slow enough to measure)
        let long_service_name = format!("service-{}", "a".repeat(1000));
        let start = Instant::now();
        let _result = Validator::validate_service_name(&long_service_name);
        let elapsed = start.elapsed();

        // Should handle large input without excessive delay (but may pass or fail validation)
        assert!(
            elapsed < Duration::from_millis(50),
            "Large service name validation too slow: {elapsed:?}"
        );

        // Test very long URL
        let long_url = format!("http://example.com/{}", "path/".repeat(1000));
        let start = Instant::now();
        let _result = Validator::validate_rpc_url(&long_url);
        let elapsed = start.elapsed();

        // Should handle large URL without excessive delay
        assert!(
            elapsed < Duration::from_millis(50),
            "Large URL validation too slow: {elapsed:?}"
        );
    }

    #[test]
    fn test_validation_error_message_performance() {
        // Test that error message formatting is fast
        let start = Instant::now();

        for i in 0..1000 {
            let error = ConfigError::invalid_value(
                "test_key",
                &format!("test_value_{i}"),
                "Test validation message",
            );
            let _message = format!("{error}");
        }

        let elapsed = start.elapsed();

        // Error formatting should be fast
        assert!(
            elapsed < Duration::from_millis(100),
            "Error formatting too slow: {elapsed:?}"
        );
    }
}

#[cfg(test)]
mod stress_tests {
    use aggsandbox::config::Config;
    use aggsandbox::validation::Validator;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{Duration, Instant};

    #[test]
    fn test_concurrent_config_loading() {
        // Test that multiple threads can load config simultaneously
        let start = Instant::now();
        let results = Arc::new(Mutex::new(Vec::new()));

        let handles: Vec<_> = (0..8)
            .map(|_| {
                let results = Arc::clone(&results);
                thread::spawn(move || {
                    for _ in 0..25 {
                        let config_result = Config::load();
                        results.lock().unwrap().push(config_result.is_ok());
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let elapsed = start.elapsed();
        let results = results.lock().unwrap();

        // All config loads should succeed
        assert_eq!(results.len(), 200); // 8 threads * 25 iterations
        assert!(results.iter().all(|&success| success));

        // Should complete in reasonable time
        assert!(
            elapsed < Duration::from_secs(5),
            "Concurrent config loading too slow: {elapsed:?}"
        );
    }

    #[test]
    fn test_validation_under_load() {
        // Test validation functions under high concurrent load
        let start = Instant::now();
        let error_count = Arc::new(Mutex::new(0));

        let handles: Vec<_> = (0..10)
            .map(|_thread_id| {
                let error_count = Arc::clone(&error_count);
                thread::spawn(move || {
                    let test_data = vec![
                        ("anvil-l1", true),
                        ("anvil-l2", true),
                        ("invalid-chain", false),
                        ("", false),
                    ];

                    for _ in 0..100 {
                        for (chain, should_succeed) in &test_data {
                            let result = Validator::validate_chain(chain);
                            if result.is_ok() != *should_succeed {
                                *error_count.lock().unwrap() += 1;
                            }
                        }
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let elapsed = start.elapsed();
        let errors = *error_count.lock().unwrap();

        // No validation errors should occur
        assert_eq!(errors, 0, "Validation errors under load: {errors}");

        // Should complete in reasonable time
        assert!(
            elapsed < Duration::from_secs(2),
            "Validation under load too slow: {elapsed:?}"
        );
    }

    #[test]
    fn test_rapid_error_creation_and_formatting() {
        // Test rapid creation and formatting of error messages
        use aggsandbox::error::{AggSandboxError, ApiError, ConfigError};

        let start = Instant::now();

        let handles: Vec<_> = (0..4)
            .map(|thread_id| {
                thread::spawn(move || {
                    for i in 0..500 {
                        // Create various error types
                        let config_err =
                            ConfigError::missing_required(&format!("VAR_{thread_id}_{i}"));
                        let api_err =
                            ApiError::network_error(&format!("Network error {thread_id}_{i}"));

                        // Convert to top-level errors
                        let sandbox_err1 = AggSandboxError::Config(config_err);
                        let sandbox_err2 = AggSandboxError::Api(api_err);

                        // Format error messages
                        let _msg1 = format!("{sandbox_err1}");
                        let _msg2 = format!("{sandbox_err2}");
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let elapsed = start.elapsed();

        // Should handle rapid error creation without issues
        assert!(
            elapsed < Duration::from_secs(1),
            "Rapid error creation too slow: {elapsed:?}"
        );
    }

    #[test]
    fn test_batch_validation_scalability() {
        // Test batch validation with increasing sizes
        let sizes = vec![10, 50, 100, 500, 1000];

        for size in sizes {
            let network_ids: Vec<u64> = (0..size)
                .map(|i| if i % 6 == 0 { 1 } else { 1101 })
                .collect();

            let start = Instant::now();
            let result =
                Validator::validate_batch(network_ids, |&id| Validator::validate_network_id(id));
            let elapsed = start.elapsed();

            // All valid network IDs should pass
            assert!(result.is_ok(), "Batch validation failed for size {size}");

            // Should scale reasonably (linear or better)
            let max_expected = Duration::from_millis(size as u64 / 10 + 50);
            assert!(
                elapsed < max_expected,
                "Batch validation for size {size} too slow: {elapsed:?} (expected < {max_expected:?})"
            );
        }
    }

    #[test]
    fn test_resource_cleanup_under_stress() {
        // Test that resources are properly cleaned up under stress
        let start = Instant::now();

        for _ in 0..100 {
            // Create many short-lived validation operations
            let addresses = (0..100)
                .map(|i| {
                    format!("0x{i:040x}") // Generate valid-format addresses
                })
                .collect::<Vec<_>>();

            for addr in addresses {
                let _result = Validator::validate_ethereum_address(&addr);
            }

            // Force some garbage collection pressure
            let _large_vec: Vec<u8> = vec![0; 10000];
        }

        let elapsed = start.elapsed();

        // Should complete without excessive memory usage or time
        assert!(
            elapsed < Duration::from_secs(3),
            "Resource cleanup stress test too slow: {elapsed:?}"
        );
    }
}

/// Benchmarking utilities for performance analysis
#[cfg(test)]
mod benchmarks {
    use aggsandbox::validation::Validator;
    use std::time::{Duration, Instant};

    /// Simple benchmark runner for consistent measurement
    pub fn benchmark<F, R>(name: &str, iterations: usize, mut operation: F) -> (Duration, Vec<R>)
    where
        F: FnMut() -> R,
    {
        println!("Running benchmark: {name} ({iterations} iterations)");

        let start = Instant::now();
        let mut results = Vec::with_capacity(iterations);

        for _ in 0..iterations {
            results.push(operation());
        }

        let elapsed = start.elapsed();
        let avg_per_op = elapsed / iterations as u32;

        println!("  Total time: {elapsed:?}");
        println!("  Average per operation: {avg_per_op:?}");
        println!(
            "  Operations per second: {:.0}",
            1_000_000_000.0 / avg_per_op.as_nanos() as f64
        );

        (elapsed, results)
    }

    #[test]
    fn benchmark_all_validations() {
        // Benchmark all validation functions for comparison

        let (elapsed, results) = benchmark("Chain validation", 10000, || {
            Validator::validate_chain("anvil-l1")
        });
        assert!(results.iter().all(|r| r.is_ok()));
        assert!(elapsed < Duration::from_millis(100));

        let (elapsed, results) = benchmark("Network ID validation", 10000, || {
            Validator::validate_network_id(1)
        });
        assert!(results.iter().all(|r| r.is_ok()));
        assert!(elapsed < Duration::from_millis(50));

        let (elapsed, results) = benchmark("Address validation", 10000, || {
            Validator::validate_ethereum_address("0x742d35cc6965c592342c6c16fb8eaeb90a23b5c0")
        });
        assert!(results.iter().all(|r| r.is_ok()));
        assert!(elapsed < Duration::from_millis(200));

        let (elapsed, results) = benchmark("URL validation", 1000, || {
            Validator::validate_rpc_url("http://localhost:8545")
        });
        assert!(results.iter().all(|r| r.is_ok()));
        assert!(elapsed < Duration::from_millis(500));
    }
}
