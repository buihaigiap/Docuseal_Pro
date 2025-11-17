use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use chrono::Utc;

use crate::database::connection::DbPool;
use crate::database::queries::SubmitterQueries;
use crate::services::email::EmailService;

#[derive(Clone)]
pub struct ReminderQueue {
    db_pool: Arc<Mutex<DbPool>>,
    email_service: Arc<EmailService>,
    base_url: String,
}

impl ReminderQueue {
    pub fn new(db_pool: Arc<Mutex<DbPool>>, email_service: EmailService, base_url: String) -> Self {
        Self {
            db_pool,
            email_service: Arc::new(email_service),
            base_url,
        }
    }

    /// Main background task that continuously checks and sends reminders
    pub async fn start_processing(&self) {
        println!("🔔 Starting reminder queue processor...");
        
        loop {
            if let Err(e) = self.process_pending_reminders().await {
                eprintln!("❌ Error processing reminders: {}", e);
            }
            
            // Check every 5 minutes
            sleep(Duration::from_secs(300)).await;
        }
    }

    /// Process all pending reminders
    async fn process_pending_reminders(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let db_guard = self.db_pool.lock().await;
        let submitters = SubmitterQueries::get_pending_reminders(&*db_guard).await?;
        
        if submitters.is_empty() {
            return Ok(());
        }

        println!("📧 Found {} submitters to check for reminders", submitters.len());
        
        for submitter in submitters {
            // Parse reminder config
            let reminder_config = match submitter.reminder_config.as_ref() {
                Some(config_json) => {
                    match serde_json::from_value::<crate::models::submitter::ReminderConfig>(config_json.clone()) {
                        Ok(config) => config,
                        Err(e) => {
                            eprintln!("Failed to parse reminder config for submitter {}: {}", submitter.id, e);
                            continue;
                        }
                    }
                },
                None => continue, // No reminder config, skip
            };

            // Calculate minutes since creation (for testing - using minutes instead of hours)
            let now = Utc::now();
            let minutes_since_created = (now - submitter.created_at).num_minutes();
            
            // Determine which reminder to send based on time elapsed
            let reminder_to_send = if submitter.reminder_count == 0 {
                // First reminder - after 1 minute
                if minutes_since_created >= reminder_config.first_reminder_hours as i64 {
                    Some(1)
                } else {
                    None
                }
            } else if submitter.reminder_count == 1 {
                // Second reminder - after 2 minutes
                if minutes_since_created >= reminder_config.second_reminder_hours as i64 {
                    Some(2)
                } else {
                    None
                }
            } else if submitter.reminder_count == 2 {
                // Third reminder - after 3 minutes
                if minutes_since_created >= reminder_config.third_reminder_hours as i64 {
                    Some(3)
                } else {
                    None
                }
            } else {
                None // Already sent all 3 reminders
            };

            if let Some(reminder_number) = reminder_to_send {
                // Check if we should send this reminder (not sent too recently)
                if let Some(last_sent) = submitter.last_reminder_sent_at {
                    let hours_since_last = (now - last_sent).num_hours();
                    // Don't send if we sent a reminder less than 1 hour ago (prevent spam)
                    if hours_since_last < 1 {
                        continue;
                    }
                }

                // Get template name for email
                let template_name = match crate::database::queries::TemplateQueries::get_template_by_id(
                    &*db_guard, 
                    submitter.template_id
                ).await {
                    Ok(Some(template)) => template.name,
                    _ => format!("Document #{}", submitter.template_id),
                };

                // Construct signature link
                let signature_link = format!("{}/s/{}", self.base_url, submitter.token);

                // Send reminder email
                println!(
                    "📨 Sending reminder #{} to {} ({}) for template '{}'", 
                    reminder_number, 
                    submitter.name, 
                    submitter.email,
                    template_name
                );

                match self.email_service.send_signature_reminder(
                    &submitter.email,
                    &submitter.name,
                    &template_name,
                    &signature_link,
                    reminder_number,
                ).await {
                    Ok(_) => {
                        // Update reminder status
                        if let Err(e) = SubmitterQueries::update_reminder_sent(&*db_guard, submitter.id).await {
                            eprintln!("Failed to update reminder status for submitter {}: {}", submitter.id, e);
                        } else {
                            println!("✅ Reminder #{} sent successfully to {}", reminder_number, submitter.email);
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to send reminder to {}: {}", submitter.email, e);
                    }
                }

                // Small delay between emails to avoid overwhelming SMTP server
                sleep(Duration::from_millis(500)).await;
            }
        }

        Ok(())
    }
}
