use std::env;
use sqlx::{PgPool, Row};
use chrono::{Utc, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderConfig {
    #[serde(default = "default_first_reminder")]
    pub first_reminder_hours: i32,
    #[serde(default = "default_second_reminder")]
    pub second_reminder_hours: i32,
    #[serde(default = "default_third_reminder")]
    pub third_reminder_hours: i32,
}

fn default_first_reminder() -> i32 { 24 }
fn default_second_reminder() -> i32 { 72 }
fn default_third_reminder() -> i32 { 168 }

#[derive(Debug)]
struct PendingReminder {
    id: i64,
    name: String,
    email: String,
    status: String,
    reminder_count: i32,
    created_at: DateTime<Utc>,
    last_reminder_sent_at: Option<DateTime<Utc>>,
    reminder_config: Option<serde_json::Value>,
    hours_since_created: i64,
    next_reminder_in_hours: Option<i64>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await?;

    let rows = sqlx::query(
        r#"
        SELECT s.id, s.name, s.email, s.status, s.reminder_count, s.created_at, s.last_reminder_sent_at, s.reminder_config
        FROM submitters s
        WHERE s.status IN ('pending', 'sent', 'viewed')
          AND s.reminder_config IS NOT NULL
          AND s.reminder_count < 3
        ORDER BY s.created_at
        "#
    )
    .fetch_all(&pool)
    .await?;

    let now = Utc::now();
    let mut pending_reminders = Vec::new();

    for row in rows {
        let id: i64 = row.get(0);
        let name: String = row.get(1);
        let email: String = row.get(2);
        let status: String = row.get(3);
        let reminder_count: i32 = row.get(4);
        let created_at: DateTime<Utc> = row.get(5);
        let last_reminder_sent_at: Option<DateTime<Utc>> = row.get(6);
        let reminder_config: Option<serde_json::Value> = row.get(7);

        let hours_since_created = (now - created_at).num_hours();

        let next_reminder_in_hours = if let Some(config) = &reminder_config {
            if let Ok(config) = serde_json::from_value::<ReminderConfig>(config.clone()) {
                match reminder_count {
                    0 => Some((config.first_reminder_hours as i64) - hours_since_created),
                    1 => Some((config.second_reminder_hours as i64) - hours_since_created),
                    2 => Some((config.third_reminder_hours as i64) - hours_since_created),
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        };

        pending_reminders.push(PendingReminder {
            id,
            name,
            email,
            status,
            reminder_count,
            created_at,
            last_reminder_sent_at,
            reminder_config,
            hours_since_created,
            next_reminder_in_hours,
        });
    }

    println!("📋 Danh sách submitters sắp nhận reminder:");
    println!("{:<5} {:<20} {:<30} {:<10} {:<15} {:<20} {:<20}",
             "ID", "Tên", "Email", "Status", "Reminder Count", "Giờ đã tạo", "Giờ đến reminder tiếp");

    for reminder in pending_reminders {
        let next_in = match reminder.next_reminder_in_hours {
            Some(h) if h > 0 => format!("{}h", h),
            Some(h) if h <= 0 => "Sẵn sàng".to_string(),
            _ => "N/A".to_string(),
        };

        println!("{:<5} {:<20} {:<30} {:<10} {:<15} {:<20} {:<20}",
                 reminder.id,
                 reminder.name.chars().take(19).collect::<String>(),
                 reminder.email.chars().take(29).collect::<String>(),
                 reminder.status,
                 reminder.reminder_count,
                 reminder.hours_since_created,
                 next_in);
    }

    Ok(())
}