use airs_mcp::base::jsonrpc::concurrent::{ConcurrentProcessor, ProcessorConfig};

#[tokio::test]
async fn test_concurrent_module_integration() {
    let config = ProcessorConfig {
        worker_count: 2,
        queue_capacity: 10,
        ..Default::default()
    };

    let mut processor = ConcurrentProcessor::new(config);

    // Test creation
    assert!(!processor.is_running());

    // Test start
    processor.start().await.unwrap();
    assert!(processor.is_running());

    // Test shutdown
    processor.shutdown().await.unwrap();
    assert!(!processor.is_running());

    println!("Concurrent module integration test passed!");
}
