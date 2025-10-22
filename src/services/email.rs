use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::env;

#[derive(Clone)]
pub struct EmailService {
    smtp_host: String,
    smtp_port: u16,
    smtp_username: String,
    smtp_password: String,
    from_email: String,
    from_name: String,
    use_tls: bool,
    test_mode: bool,
}

impl EmailService {
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let smtp_host = env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_string());
        let smtp_port: u16 = env::var("SMTP_PORT")
            .unwrap_or_else(|_| "587".to_string())
            .parse()
            .unwrap_or(587);
        let smtp_username = env::var("SMTP_USERNAME")?;
        let smtp_password = env::var("SMTP_PASSWORD")?;
        let from_email = env::var("FROM_EMAIL")?;
        let from_name = env::var("FROM_NAME").unwrap_or_else(|_| "DocuSeal Pro".to_string());
        let use_tls: bool = env::var("SMTP_USE_TLS")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);
        let test_mode: bool = env::var("EMAIL_TEST_MODE")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        Ok(Self {
            smtp_host,
            smtp_port,
            smtp_username,
            smtp_password,
            from_email,
            from_name,
            use_tls,
            test_mode,
        })
    }

    pub async fn send_signature_request(
        &self,
        to_email: &str,
        to_name: &str,
        submission_name: &str,
        signature_link: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        if self.test_mode {
            println!("TEST MODE: Would send email to {} ({}) with link: {}", to_email, to_name, signature_link);
            return Ok(());
        }

        let subject = format!("Yêu cầu ký tài liệu: {}", submission_name);

        let html_body = format!(
            r#"
<!DOCTYPE html>
<html lang="vi">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Yêu cầu ký tài liệu</title>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 600px;
            margin: 0 auto;
            background-color: #f8f9fa;
            padding: 20px;
        }}
        .container {{
            background: white;
            padding: 30px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }}
        .header {{
            text-align: center;
            margin-bottom: 30px;
        }}
        .header h1 {{
            color: #007bff;
            margin-bottom: 10px;
        }}
        .content {{
            margin-bottom: 30px;
        }}
        .button {{
            display: inline-block;
            padding: 12px 24px;
            background: linear-gradient(135deg, #007bff 0%, #0056b3 100%);
            color: white;
            text-decoration: none;
            border-radius: 6px;
            font-weight: bold;
            text-align: center;
            margin: 20px 0;
        }}
        .footer {{
            margin-top: 30px;
            padding-top: 20px;
            border-top: 1px solid #e9ecef;
            font-size: 14px;
            color: #6c757d;
            text-align: center;
        }}
        .warning {{
            background: #fff3cd;
            border: 1px solid #ffeaa7;
            color: #856404;
            padding: 15px;
            border-radius: 5px;
            margin: 20px 0;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>📝 Yêu cầu ký tài liệu</h1>
            <p>Xin chào <strong>{}</strong>,</p>
        </div>

        <div class="content">
            <p>Bạn nhận được yêu cầu ký tài liệu từ hệ thống <strong>DocuSeal Pro</strong>.</p>

            <div class="warning">
                <strong>Quan trọng:</strong> Link ký tài liệu này chỉ có hiệu lực trong thời gian giới hạn.
                Vui lòng hoàn thành việc ký trong thời gian sớm nhất có thể.
            </div>

            <p><strong>Tên tài liệu:</strong> {}</p>

            <p>Vui lòng nhấp vào nút bên dưới để truy cập và ký tài liệu:</p>

            <a href="{}" class="button">📝 Truy cập và ký tài liệu</a>

            <p>Nếu nút trên không hoạt động, bạn có thể sao chép và dán link sau vào trình duyệt:</p>
            <p style="word-break: break-all; background: #f8f9fa; padding: 10px; border-radius: 5px; font-family: monospace;">{}</p>
        </div>

        <div class="footer">
            <p>Email này được gửi tự động từ hệ thống DocuSeal Pro.</p>
            <p>Nếu bạn không mong muốn nhận email này, vui lòng bỏ qua.</p>
            <p>&copy; 2025 DocuSeal Pro. Tất cả quyền được bảo lưu.</p>
        </div>
    </div>
</body>
</html>
            "#,
            to_name, submission_name, signature_link, signature_link
        );

        let text_body = format!(
            "Xin chào {},\n\n\
            Bạn nhận được yêu cầu ký tài liệu '{}' từ hệ thống DocuSeal Pro.\n\n\
            Vui lòng truy cập link sau để ký tài liệu:\n\
            {}\n\n\
            Link này chỉ có hiệu lực trong thời gian giới hạn.\n\n\
            Trân trọng,\n\
            DocuSeal Pro",
            to_name, submission_name, signature_link
        );

        let email = Message::builder()
            .from(format!("{} <{}>", self.from_name, self.from_email).parse()?)
            .to(format!("{} <{}>", to_name, to_email).parse()?)
            .subject(subject)
            .multipart(
                lettre::message::MultiPart::alternative()
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(lettre::message::header::ContentType::TEXT_PLAIN)
                            .body(text_body),
                    )
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(lettre::message::header::ContentType::TEXT_HTML)
                            .body(html_body),
                    ),
            )?;

        let creds = Credentials::new(self.smtp_username.clone(), self.smtp_password.clone());

        let mailer = if self.use_tls {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.smtp_host)?
                .credentials(creds)
                .build()
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&self.smtp_host)?
                .credentials(creds)
                .build()
        };

        mailer.send(email).await?;
        println!("Email sent successfully to: {}", to_email);

        Ok(())
    }

    pub async fn send_user_activation_email(
        &self,
        to_email: &str,
        to_name: &str,
        activation_link: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        if self.test_mode {
            println!("TEST MODE: Would send activation email to {} ({}) with link: {}", to_email, to_name, activation_link);
            return Ok(());
        }

        let subject = "Kích hoạt tài khoản DocuSeal Pro".to_string();

        let html_body = format!(
            r#"
<!DOCTYPE html>
<html lang="vi">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Kích hoạt tài khoản</title>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 600px;
            margin: 0 auto;
            background-color: #f8f9fa;
            padding: 20px;
        }}
        .container {{
            background: white;
            padding: 30px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }}
        .header {{
            text-align: center;
            margin-bottom: 30px;
        }}
        .header h1 {{
            color: #007bff;
            margin-bottom: 10px;
        }}
        .content {{
            margin-bottom: 30px;
        }}
        .button {{
            display: inline-block;
            padding: 12px 24px;
            background-color: #007bff;
            color: white;
            text-decoration: none;
            border-radius: 5px;
            font-weight: bold;
        }}
        .footer {{
            margin-top: 30px;
            padding-top: 20px;
            border-top: 1px solid #eee;
            font-size: 12px;
            color: #666;
            text-align: center;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Chào mừng đến với DocuSeal Pro!</h1>
        </div>
        <div class="content">
            <p>Xin chào <strong>{}</strong>,</p>
            <p>Tài khoản của bạn đã được tạo thành công. Để kích hoạt tài khoản và bắt đầu sử dụng DocuSeal Pro, vui lòng nhấp vào nút bên dưới:</p>
            <p style="text-align: center;">
                <a href="{}" class="button">Kích hoạt tài khoản</a>
            </p>
            <p>Nếu nút không hoạt động, bạn có thể sao chép và dán liên kết sau vào trình duyệt:</p>
            <p><a href="{}">{}</a></p>
            <p>Liên kết này sẽ hết hạn sau 24 giờ.</p>
        </div>
        <div class="footer">
            <p>Email này được gửi tự động từ hệ thống DocuSeal Pro.</p>
            <p>Nếu bạn không mong muốn nhận email này, vui lòng bỏ qua.</p>
        </div>
    </div>
</body>
</html>
            "#,
            to_name, activation_link, activation_link, activation_link
        );

        let text_body = format!(
            "Xin chào {},\n\nTài khoản của bạn đã được tạo. Để kích hoạt, truy cập: {}\n\nLiên kết hết hạn sau 24 giờ.\n\nDocuSeal Pro",
            to_name, activation_link
        );

        let email = Message::builder()
            .from(format!("{} <{}>", self.from_name, self.from_email).parse()?)
            .to(format!("{} <{}>", to_name, to_email).parse()?)
            .subject(subject)
            .multipart(
                lettre::message::MultiPart::alternative()
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(lettre::message::header::ContentType::TEXT_PLAIN)
                            .body(text_body),
                    )
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(lettre::message::header::ContentType::TEXT_HTML)
                            .body(html_body),
                    ),
            )?;

        let creds = Credentials::new(self.smtp_username.clone(), self.smtp_password.clone());

        let mailer = if self.use_tls {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.smtp_host)?
                .credentials(creds)
                .build()
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&self.smtp_host)?
                .credentials(creds)
                .build()
        };

        mailer.send(email).await?;
        println!("Activation email sent successfully to: {}", to_email);

        Ok(())
    }

    pub async fn send_signature_completed(
        &self,
        to_email: &str,
        to_name: &str,
        submission_name: &str,
        submitter_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        if self.test_mode {
            println!("TEST MODE: Would send completion email to {} ({}) for submission: {}", to_email, to_name, submission_name);
            return Ok(());
        }

        let subject = format!("Hoàn thành ký tài liệu: {}", submission_name);

        let html_body = format!(
            r#"
<!DOCTYPE html>
<html lang="vi">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Hoàn thành ký tài liệu</title>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 600px;
            margin: 0 auto;
            background-color: #f8f9fa;
            padding: 20px;
        }}
        .container {{
            background: white;
            padding: 30px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }}
        .header {{
            text-align: center;
            margin-bottom: 30px;
        }}
        .header h1 {{
            color: #28a745;
            margin-bottom: 10px;
        }}
        .success-icon {{
            font-size: 48px;
            color: #28a745;
            margin-bottom: 20px;
        }}
        .content {{
            margin-bottom: 30px;
        }}
        .footer {{
            margin-top: 30px;
            padding-top: 20px;
            border-top: 1px solid #e9ecef;
            font-size: 14px;
            color: #6c757d;
            text-align: center;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="success-icon">✅</div>
            <h1>Hoàn thành ký tài liệu</h1>
            <p>Xin chào <strong>{}</strong>,</p>
        </div>

        <div class="content">
            <p>Chúng tôi xin thông báo rằng tài liệu <strong>"{}"</strong> đã được ký thành công bởi <strong>{}</strong>.</p>

            <p>Tài liệu đã được xử lý và lưu trữ an toàn trong hệ thống DocuSeal Pro.</p>

            <p>Cảm ơn bạn đã sử dụng dịch vụ của chúng tôi!</p>
        </div>

        <div class="footer">
            <p>Email này được gửi tự động từ hệ thống DocuSeal Pro.</p>
            <p>&copy; 2025 DocuSeal Pro. Tất cả quyền được bảo lưu.</p>
        </div>
    </div>
</body>
</html>
            "#,
            to_name, submission_name, submitter_name
        );

        let text_body = format!(
            "Xin chào {},\n\n\
            Tài liệu '{}' đã được ký thành công bởi {}.\n\n\
            Tài liệu đã được lưu trữ an toàn trong hệ thống.\n\n\
            Cảm ơn bạn đã sử dụng dịch vụ DocuSeal Pro!\n\n\
            Trân trọng,\n\
            DocuSeal Pro",
            to_name, submission_name, submitter_name
        );

        let email = Message::builder()
            .from(format!("{} <{}>", self.from_name, self.from_email).parse()?)
            .to(format!("{} <{}>", to_name, to_email).parse()?)
            .subject(subject)
            .multipart(
                lettre::message::MultiPart::alternative()
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(lettre::message::header::ContentType::TEXT_PLAIN)
                            .body(text_body),
                    )
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(lettre::message::header::ContentType::TEXT_HTML)
                            .body(html_body),
                    ),
            )?;

        let creds = Credentials::new(self.smtp_username.clone(), self.smtp_password.clone());

        let mailer = if self.use_tls {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.smtp_host)?
                .credentials(creds)
                .build()
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&self.smtp_host)?
                .credentials(creds)
                .build()
        };

        mailer.send(email).await?;
        println!("Completion email sent successfully to: {}", to_email);

        Ok(())
    }
}