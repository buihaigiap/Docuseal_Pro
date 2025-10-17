use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

// Simple test without database
#[derive(Debug, Clone)]
struct TestPayment {
    user_id: i64,
    amount: f64,
}

struct TestQueue {
    queue: Arc<Mutex<VecDeque<TestPayment>>>,
}

impl TestQueue {
    fn new() -> Self {
        TestQueue {
            queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    async fn enqueue(&self, payment: TestPayment) {
        let mut q = self.queue.lock().await;
        q.push_back(payment.clone());
        println!("Payment enqueued: user_id={}, amount={}", payment.user_id, payment.amount);
    }

    async fn process_parallel(&self, max_concurrent: usize) {
        use futures::stream::{self, StreamExt};

        loop {
            // Get batch payments
            let payments_batch = {
                let mut q = self.queue.lock().await;
                let mut batch = Vec::new();
                for _ in 0..max_concurrent {
                    if let Some(payment) = q.pop_front() {
                        batch.push(payment);
                    } else {
                        break;
                    }
                }
                batch
            };

            if payments_batch.is_empty() {
                sleep(Duration::from_millis(10)).await;
                continue;
            }

            // Process batch in parallel
            let results: Vec<()> = stream::iter(payments_batch)
                .map(|payment| async move {
                    println!("Processing payment: user_id={}, amount={}", payment.user_id, payment.amount);
                    // Simulate processing
                    sleep(Duration::from_millis(100)).await;
                    println!("Payment processed: user_id={}", payment.user_id);
                })
                .buffer_unordered(max_concurrent)
                .collect()
                .await;

            println!("Processed batch of {} payments", results.len());
        }
    }

    // Adaptive batch processing - tự động chia batch dựa trên queue size
    async fn process_adaptive(&self) {
        use futures::stream::{self, StreamExt};

        loop {
            // Lấy toàn bộ queue size để quyết định cách chia batch
            let queue_size = {
                let q = self.queue.lock().await;
                q.len()
            };

            if queue_size == 0 {
                sleep(Duration::from_millis(10)).await;
                continue;
            }

            // Tính toán số batch và size dựa trên queue size
            let (num_batches, batch_size) = self.calculate_batch_strategy(queue_size);

            println!("Queue size: {}, using {} batches of ~{} items each", queue_size, num_batches, batch_size);

            // Chia queue thành các batch
            let mut batch_tasks = Vec::new();

            for batch_idx in 0..num_batches {
                // Lấy batch từ queue
                let batch_payments = {
                    let mut q = self.queue.lock().await;
                    let mut batch = Vec::new();
                    let actual_batch_size = if batch_idx == num_batches - 1 {
                        // Batch cuối cùng lấy tất cả còn lại
                        batch_size + (queue_size % batch_size)
                    } else {
                        batch_size
                    };

                    for _ in 0..actual_batch_size {
                        if let Some(payment) = q.pop_front() {
                            batch.push(payment);
                        } else {
                            break;
                        }
                    }
                    batch
                };

                if batch_payments.is_empty() {
                    break;
                }

                // Tạo task để xử lý batch này
                let batch_task = tokio::spawn(async move {
                    println!("Processing batch {} with {} payments", batch_idx + 1, batch_payments.len());

                    // Xử lý payments trong batch song song
                    let results: Vec<()> = stream::iter(batch_payments)
                        .map(|payment| async move {
                            println!("  -> Processing payment: user_id={}, amount={}",
                                   payment.user_id, payment.amount);
                            // Simulate DB operation
                            sleep(Duration::from_millis(100)).await;
                            println!("  -> Payment processed: user_id={}", payment.user_id);
                        })
                        .buffer_unordered(10) // Max 10 concurrent trong mỗi batch
                        .collect()
                        .await;

                    println!("Batch {} completed: {} payments processed", batch_idx + 1, results.len());
                    results.len()
                });

                batch_tasks.push(batch_task);
            }

            // Chờ các batch hoàn thành với giới hạn concurrency
            let completed_batches = futures::future::join_all(batch_tasks).await;
            let mut total_processed = 0;
            for result in completed_batches {
                match result {
                    Ok(count) => total_processed += count,
                    Err(e) => println!("Batch processing error: {:?}", e),
                }
            }

            println!("Adaptive processing cycle completed: {} payments processed across {} batches",
                    total_processed, num_batches);
        }
    }

    // Tính toán chiến lược chia batch dựa trên queue size
    fn calculate_batch_strategy(&self, queue_size: usize) -> (usize, usize) {
        match queue_size {
            0 => (0, 0),
            1..=10 => {
                // Queue nhỏ: 1 batch, process tất cả
                (1, queue_size)
            },
            11..=50 => {
                // Queue trung bình: 2-3 batch, mỗi batch ~10-15 items
                let num_batches = 2 + (queue_size / 25).min(1); // 2-3 batches
                let batch_size = (queue_size + num_batches - 1) / num_batches; // Round up
                (num_batches, batch_size)
            },
            51..=200 => {
                // Queue lớn: nhiều batch nhỏ hơn, mỗi batch ~10-15 items
                let num_batches = 4 + (queue_size / 50).min(4); // 4-8 batches
                let batch_size = (queue_size + num_batches - 1) / num_batches;
                (num_batches, batch_size)
            },
            _ => {
                // Queue rất lớn: nhiều batch nhỏ để tránh overload
                let num_batches = 10 + (queue_size / 100).min(10); // 10-20 batches
                let batch_size = (queue_size + num_batches - 1) / num_batches;
                (num_batches, batch_size.min(20)) // Max 20 items per batch
            }
        }
    }

    async fn get_queue_size(&self) -> usize {
        let q = self.queue.lock().await;
        q.len()
    }
}

#[tokio::main]
async fn main() {
    println!("Testing Payment Queue - Adaptive Batch Processing");
    println!("================================================");

    // Test 1: Fixed batch size (version cũ)
    println!("\n=== TEST 1: Fixed Batch Size (max_concurrent=3) ===");
    test_fixed_batch().await;

    // Test 2: Adaptive batch processing (version mới)
    println!("\n=== TEST 2: Adaptive Batch Processing ===");
    test_adaptive_batch().await;

    println!("\n🎉 All tests completed!");
}

async fn test_fixed_batch() {
    println!("Testing fixed batch processing...");

    let queue = Arc::new(TestQueue::new());

    // Start processor với fixed batch size
    let processor_queue = queue.clone();
    tokio::spawn(async move {
        processor_queue.process_parallel(3).await;
    });

    // Enqueue 8 payments
    println!("Enqueuing 8 payments...");
    for i in 1..=8 {
        let payment = TestPayment {
            user_id: i as i64,
            amount: (i * 10) as f64,
        };
        queue.enqueue(payment).await;
        sleep(Duration::from_millis(20)).await;
    }

    // Wait for processing
    sleep(Duration::from_secs(3)).await;

    let final_size = queue.get_queue_size().await;
    println!("Fixed batch test completed. Final queue size: {}", final_size);
}

async fn test_adaptive_batch() {
    println!("Testing adaptive batch processing...");

    let queue = Arc::new(TestQueue::new());

    // Start adaptive processor
    let processor_queue = queue.clone();
    tokio::spawn(async move {
        processor_queue.process_adaptive().await;
    });

    // Test với queue size khác nhau
    let test_sizes = vec![5, 25, 75, 150];

    for (test_idx, size) in test_sizes.iter().enumerate() {
        println!("\n--- Sub-test {}: Queue size {} ---", test_idx + 1, size);

        // Enqueue payments
        for i in 1..=*size {
            let payment = TestPayment {
                user_id: (test_idx * 1000 + i) as i64,
                amount: (i * 5) as f64,
            };
            queue.enqueue(payment).await;
        }

        println!("Enqueued {} payments, waiting for adaptive processing...", size);

        // Wait for processing (adjust time based on size)
        let wait_time = match size {
            1..=10 => 2,
            11..=50 => 4,
            51..=100 => 6,
            _ => 8,
        };
        sleep(Duration::from_secs(wait_time)).await;

        let final_size = queue.get_queue_size().await;
        println!("Sub-test {} completed. Final queue size: {}", test_idx + 1, final_size);
    }

    println!("Adaptive batch test completed!");
}