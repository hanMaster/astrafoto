use crate::Result;
use crate::stuff::config::config;
use crate::stuff::order::Order;
use async_mailer::{IntoMessage, Mailer, SmtpMailer};
use log::info;

#[derive(Default)]
pub struct Email;

impl Email {
    pub async fn send(&self, subject: &str, content: String) -> Result<()> {
        let mailer: SmtpMailer = SmtpMailer::new(
            config().SMTP_SERVER.clone(),
            config().SMTP_PORT,
            async_mailer::SmtpInvalidCertsPolicy::Deny,
            config().SENDER_EMAIL.clone(),
            async_mailer::SecretString::from(config().SENDER_PASS.clone()),
        );

        let message = async_mailer::MessageBuilder::new()
            .from(("From Astrafoto-worker", config().SENDER_EMAIL.as_str()))
            .to(config().RECEIVER_EMAIL.as_str())
            .subject(subject)
            .html_body(content)
            .into_message()?;

        mailer.send_mail(message).await?;
        Ok(())
    }

    pub async fn send_new_order(&self, order: Order, order_id: String) -> Result<()> {
        info!("Sending email with {}", order_id);
        let content = self.prepare_new_order_content(order, order_id.clone());
        self.send("Новый заказ", content).await?;
        info!("Email sent with {}", order_id);
        Ok(())
    }

    pub async fn send_state(&self, state: String) -> Result<()> {
        info!("Sending email with state {}", state);
        let content = self.prepare_state_content(state);
        self.send("Состояние инстанса WhatsApp", content).await?;
        info!("Email sent");
        Ok(())
    }

    fn prepare_new_order_content(&self, order: Order, order_id: String) -> String {
        let cnt = order.files.len() as i32;

        format!(
            r#"
        <!DOCTYPE html>
<html lang="ru">
    <head>
        <meta charset="UTF-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <title></title>
    </head>
    <body style="font-family: sans-serif">
        <h1>
            Получен заказ по
            <span style="color: rgb(12, 193, 67)">WhatsApp</span>
            № <span style="font-weight: 700" id="order-number">{}</span>
        </h1>
        <p>Телефон: <span id="phone">{}</span></p>
        <p>Имя: <span id="name">{}</span></p>

        <p>Бумага: <span id="paper">{} {}</span></p>
        <p>{}шт х {}руб = {}руб</p>
    </body>
</html>
        "#,
            order_id,
            order.phone,
            order.name,
            order.paper_size,
            order.paper_type,
            cnt,
            order.price,
            cnt * order.price,
        )
    }

    fn prepare_state_content(&self, state: String) -> String {
        format!(
            r#"
        <!DOCTYPE html>
<html lang="ru">
    <head>
        <meta charset="UTF-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <title></title>
    </head>
    <body style="font-family: sans-serif">
        <h2>
            Получено новое состояние WhatsApp инстанса: {}
        </h2>
    </body>
</html>
        "#,
            state
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[tokio::test]
    async fn send_email_test() {
        let order = Order {
            phone: "79147894556".to_string(),
            name: "Иван".to_string(),
            paper_type: "глянцевая".to_string(),
            paper_size: "10x15 - 32руб".to_string(),
            price: 15,
            files: vec!["123".to_string(), "456".to_string(), "789".to_string()],
        };
        let res = Email
            .send_new_order(order, "WA-18032025-1000".to_string())
            .await;
        match res {
            Ok(_) => println!("Sent email"),
            Err(ref e) => println!("Error sending email: {}", e),
        }
        assert!(res.is_ok());
    }
    #[tokio::test]
    async fn send_state_test() {
        let state = "Active".to_string();
        let res = Email
            .send_state(state)
            .await;
        match res {
            Ok(_) => println!("Sent email"),
            Err(ref e) => println!("Error sending email: {}", e),
        }
        assert!(res.is_ok());
    }
}
