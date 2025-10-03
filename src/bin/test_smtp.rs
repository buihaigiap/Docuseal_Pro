use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let smtp_host = env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_string());
    let smtp_port: u16 = env::var("SMTP_PORT").unwrap_or_else(|_| "587".to_string()).parse().unwrap_or(587);
    let smtp_username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set");
    let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");
    let from_email = env::var("FROM_EMAIL").expect("FROM_EMAIL must be set");

    println!("Testing SMTP connection to {}:{}", smtp_host, smtp_port);
    println!("Using username: {}", smtp_username);
    println!("From email: {}", from_email);

    let creds = Credentials::new(smtp_username.clone(), smtp_password.clone());

    let mailer = SmtpTransport::relay(&smtp_host)
        .unwrap()
        .credentials(creds)
        .build();

    let email = Message::builder()
        .from(from_email.parse().unwrap())
        .to("test@example.com".parse().unwrap())
        .subject("SMTP Test")
        .body(String::from("This is a test email to verify SMTP connection."))
        .unwrap();

    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => println!("Failed to send email: {:?}", e),
    }

    Ok(())
}