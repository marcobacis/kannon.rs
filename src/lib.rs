use base64::Engine;
use pkg::kannon::mailer::apiv1::{SendHtmlReq, SendTemplateReq, mailer_client::MailerClient};
use std::collections::HashMap;
use thiserror::Error;
use tonic::{
    Request, Status,
    metadata::MetadataKey,
    transport::{Channel, ClientTlsConfig},
};

mod pkg {
    pub mod kannon {
        pub mod mailer {
            pub mod apiv1 {
                tonic::include_proto!("pkg.kannon.mailer.apiv1");
            }
            pub mod types {
                tonic::include_proto!("pkg.kannon.mailer.types");
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error while connecting to kannon host")]
    ConnectionError(#[from] tonic::transport::Error),
    #[error("Error while sending mail")]
    SendMailError(#[from] Status),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sender {
    pub email: String,
    pub alias: String,
}

impl From<Sender> for pkg::kannon::mailer::types::Sender {
    fn from(val: Sender) -> Self {
        pkg::kannon::mailer::types::Sender {
            email: val.email,
            alias: val.alias,
        }
    }
}

/// A recipient to which to send emails
/// 
/// The recipient can also have some additional fields,
/// which are replaced when sending the email.
/// 
/// For example: 
/// ```rust
/// # use std::collections::HashMap;
/// # use kannon::Recipient;
/// let recipient = Recipient {
///     email: "test@email.com".into(),
///     fields: HashMap::from([("name".into(), "Test User".into())])
/// };
/// ```
/// 
/// When sending the email or template, if the body contains `{{ name }}`
/// it will be replaced with `Test User`
#[derive(Debug, Clone, PartialEq)]
pub struct Recipient {
    pub email: String,
    pub fields: HashMap<String, String>,
}

impl From<Recipient> for pkg::kannon::mailer::types::Recipient {
    fn from(val: Recipient) -> Self {
        pkg::kannon::mailer::types::Recipient {
            email: val.email,
            fields: val.fields,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attachment {
    pub filename: String,
    pub content: Vec<u8>,
}

impl From<Attachment> for pkg::kannon::mailer::apiv1::Attachment {
    fn from(val: Attachment) -> Self {
        pkg::kannon::mailer::apiv1::Attachment {
            filename: val.filename,
            content: val.content,
        }
    }
}

/// Kannon mail client
pub struct Kannon {
    domain: String,
    key: String,
    sender: Sender,
    client: MailerClient<Channel>,
}

impl Kannon {

    /// Instantiates the kannon client and connects to the host
    /// 
    /// # Example
    /// ```rust, no_run
    /// # use kannon::{Sender, Kannon};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), kannon::Error> {
    /// let mut kannon = Kannon::new(
    ///     "example.com".into(),
    ///     "<your key>".into(),
    ///     Sender {
    ///         email: "test@example.com".into(),
    ///         alias: "Test Sender".into(),
    ///     },
    ///     "<kannon host>".into(),
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    /// 
    /// Note: if you connect to the official kannon server, remember to put `https` in the host!
    /// 
    pub async fn new(
        domain: String,
        key: String,
        sender: Sender,
        host: String,
    ) -> Result<Self, Error> {
        // TODO manage errors
        let channel = Channel::from_shared(host.clone())
            .unwrap()
            .tls_config(ClientTlsConfig::new().with_native_roots())?
            .connect()
            .await?;

        Ok(Self {
            domain,
            key,
            sender,
            client: MailerClient::new(channel),
        })
    }

    fn get_auth_header(&self) -> String {
        let token = base64::engine::general_purpose::STANDARD
            .encode(format!("{}:{}", &self.domain, &self.key));
        format!("Basic {}", token)
    }

    pub async fn send_email(
        &mut self,
        recipients: Vec<Recipient>,
        subject: String,
        body: String,
        attachments: Vec<Attachment>,
    ) -> Result<(), Error> {
        let mut request = Request::new(SendHtmlReq {
            sender: Some(self.sender.clone().into()),
            subject,
            html: body,
            scheduled_time: None,
            recipients: recipients.into_iter().map(Recipient::into).collect(),
            attachments: attachments.into_iter().map(Attachment::into).collect(),
            global_fields: HashMap::new(),
        });

        let metadata_value = self.get_auth_header().try_into().unwrap();
        request
            .metadata_mut()
            .insert(MetadataKey::from_static("authorization"), metadata_value);

        self.client.send_html(request).await?;

        Ok(())
    }

    pub async fn send_template(
        &mut self,
        recipients: Vec<Recipient>,
        subject: String,
        template_id: String,
        attachments: Vec<Attachment>,
    ) -> Result<(), Error> {
        let mut request = Request::new(SendTemplateReq {
            sender: Some(self.sender.clone().into()),
            subject,
            template_id,
            scheduled_time: None,
            recipients: recipients.into_iter().map(Recipient::into).collect(),
            attachments: attachments.into_iter().map(Attachment::into).collect(),
            global_fields: HashMap::new(),
        });

        let metadata_value = self.get_auth_header().try_into().unwrap();
        request
            .metadata_mut()
            .insert(MetadataKey::from_static("authorization"), metadata_value);

        self.client.send_template(request).await?;

        Ok(())
    }
}
