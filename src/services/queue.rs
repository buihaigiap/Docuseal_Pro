use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use futures::stream::{self, StreamExt};

use crate::database::models::CreatePaymentRecord;
use crate::database::queries::SubscriptionQueries;
use crate::database::connection::DbPool;

#[derive(Clone)]
pub struct PaymentQueue {
    queue: Arc<Mutex<VecDeque<CreatePaymentRecord>>>,
    db_pool: Arc<Mutex<DbPool>>,
}

impl PaymentQueue {
    pub fn new(db_pool: Arc<Mutex<DbPool>>) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            db_pool,
        }
    }

    // Thêm payment vào queue
    pub async fn enqueue(&self, payment: CreatePaymentRecord) {
        let mut q = self.queue.lock().await;
        q.push_back(payment.clone());
        println!("Payment added: {:?}", payment.user_id);
    }

    // Xử lý payment song song với max_concurrent task
    pub async fn process_parallel(&self, max_concurrent: usize) {
        loop {
            // Lấy tất cả payment hiện có trong queue
            let payments_batch = {
                let mut q = self.queue.lock().await;
                let batch: Vec<_> = q.drain(..).collect(); // lấy hết queue hiện tại
                batch
            };

            if payments_batch.is_empty() {
                sleep(Duration::from_millis(100)).await; // queue rỗng, chờ
                continue;
            }

            // Xử lý batch song song, tối đa max_concurrent
            let results = stream::iter(payments_batch)
                .map(|payment| {
                    let db_pool = Arc::clone(&self.db_pool);
                    async move {
                        let user_id = payment.user_id;
                        println!("Processing payment for user {}: ${}", user_id, payment.amount_cents as f64 / 100.0);

                        let db_guard = db_pool.lock().await;
                        match SubscriptionQueries::create_payment_record(&*db_guard, payment).await {
                            Ok(record) => {
                                println!("💾 Created payment record {}", record.id);
                                Ok(record.id)
                            }
                            Err(e) => {
                                eprintln!("Failed to create payment record: {}", e);
                                Err(e)
                            }
                        }
                    }
                })
                .buffer_unordered(max_concurrent)
                .collect::<Vec<_>>()
                .await;

            // Log kết quả batch
            let success_count = results.iter().filter(|r| r.is_ok()).count();
            let total_count = results.len();
            println!("✅ Processed batch: {}/{} payments successful", success_count, total_count);

            sleep(Duration::from_millis(10)).await; // tránh quá tải CPU
        }
    }
}
