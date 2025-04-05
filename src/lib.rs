use base64::Engine;
use pkg::kannon::mailer::apiv1::{mailer_client::MailerClient, SendHtmlReq, SendTemplateReq};
use thiserror::Error;
use std::collections::HashMap;
use tonic::{
    metadata::MetadataKey, transport::{Channel, ClientTlsConfig}, Request, Status
};

pub mod pkg {
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
    SendMailError(#[from] Status)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sender {
    pub email: String,
    pub alias: String,
}

impl Into<pkg::kannon::mailer::types::Sender> for Sender {
    fn into(self) -> pkg::kannon::mailer::types::Sender {
        pkg::kannon::mailer::types::Sender {
            email: self.email,
            alias: self.alias,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Recipient {
    pub email: String,
    pub fields: HashMap<String, String>,
}

impl Into<pkg::kannon::mailer::types::Recipient> for Recipient {
    fn into(self) -> pkg::kannon::mailer::types::Recipient {
        pkg::kannon::mailer::types::Recipient {
            email: self.email,
            fields: self.fields,
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct Attachment {
    pub filename: String,
    pub content: Vec<u8>,
}

impl Into<pkg::kannon::mailer::apiv1::Attachment> for Attachment {
    fn into(self) -> pkg::kannon::mailer::apiv1::Attachment {
        pkg::kannon::mailer::apiv1::Attachment {
            filename: self.filename,
            content: self.content,
        }
    }
}

pub struct Kannon {
    domain: String,
    key: String,
    sender: Sender,
    client: MailerClient<Channel>,
}

impl Kannon {
    pub async fn new(domain: String, key: String, sender: Sender, host: String) -> Result<Self, Error> {
        // TODO manage errors
        let channel = Channel::from_shared(host.clone()).unwrap()
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
        let token = base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", &self.domain, &self.key));
        format!("Basic {}", token)
    }

    pub async fn send_email(&mut self, recipients: Vec<Recipient>, subject: String, body: String, attachments: Vec<Attachment>) -> Result<(), Error> {
        let mut request = Request::new(SendHtmlReq {
            sender: Some(self.sender.clone().into()),
            subject: subject,
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

    pub async fn send_template(&mut self, recipients: Vec<Recipient>, subject: String, template_id: String, attachments: Vec<Attachment>) -> Result<(), Error> {
        let mut request = Request::new(SendTemplateReq {
            sender: Some(self.sender.clone().into()),
            subject: subject,
            template_id: template_id,
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
