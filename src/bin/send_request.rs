use lettre::transport::smtp::authentication::Credentials;
use lettre::{message::header::ContentType, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use dotenvy::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let smtp_host = env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_string());
    let smtp_username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set");
    let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");
    let from_email = env::var("FROM_EMAIL").expect("FROM_EMAIL must be set");
    let from_name = env::var("FROM_NAME").unwrap_or_else(|_| "DocuSeal Pro".to_string());

    // target email can be passed as first arg, otherwise use the FROM_EMAIL
    let args: Vec<String> = env::args().collect();
    let to_email = args.get(1).cloned().unwrap_or_else(|| from_email.clone());
    let to_name = args.get(2).cloned().unwrap_or_else(|| "Recipient".to_string());
    let submission_name = args.get(3).cloned().unwrap_or_else(|| "Test Submission".to_string());
    let signature_link = args.get(4).cloned().unwrap_or_else(|| "http://localhost:8080/public/submitters/test-token".to_string());

    println!("Sending signature request to {} <{}>", to_name, to_email);

    let html_body = format!(
        r#"<!DOCTYPE html>
<html lang=\"vi\"> 
<head><meta charset=\"UTF-8\"></head>
<body>
<h1>📝 Yêu cầu ký tài liệu</h1>
<p>Xin chào <strong>{}</strong>,</p>
<p>Bạn nhận được yêu cầu ký tài liệu từ hệ thống <strong>DocuSeal Pro</strong>.</p>
<p><strong>Tên tài liệu:</strong> {}</p>
<p><a href=\"{}\">📝 Truy cập và ký tài liệu</a></p>
<p>Nếu không hoạt động, sao chép link sau:</p>
<pre>{}</pre>
</body>
</html>"#,
        to_name, submission_name, signature_link, signature_link
    );

    let text_body = format!(
        "Xin chào {},\n\nBạn nhận được yêu cầu ký tài liệu '{}' từ hệ thống DocuSeal Pro.\n\nTruy cập link: {}\n\nTrân trọng,\nDocuSeal Pro",
        to_name, submission_name, signature_link
    );

    let email = Message::builder()
        .from(format!("{} <{}>", from_name, from_email).parse()?)
        .to(format!("{} <{}>", to_name, to_email).parse()?)
        .subject(format!("Yêu cầu ký tài liệu: {}", submission_name))
        .multipart(
            lettre::message::MultiPart::alternative()
                .singlepart(
                    lettre::message::SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(text_body),
                )
                .singlepart(
                    lettre::message::SinglePart::builder()
                        .header(ContentType::TEXT_HTML)
                        .body(html_body),
                ),
        )?;

    let creds = Credentials::new(smtp_username.clone(), smtp_password.clone());

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_host)?
        .credentials(creds)
        .build();

    match mailer.send(email).await {
        Ok(_) => println!("Signature request email sent successfully to {}", to_email),
        Err(e) => println!("Failed to send signature request: {:?}", e),
    }

    Ok(())
}
