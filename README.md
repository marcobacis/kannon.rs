## kannon.rs: unofficial Rust client library for Kannon Email Sender

`kannon-rs` is the (unofficial) Rust client library for [Kannon](https://www.kannon.email/).

## Usage

First, instantiate the Kannon client 

```rust
let sender = Sender {
    email: "sender@kannon.dev".into(),
    alias: "Kannon".into(),
};

let mut kannon = Kannon::new(
    "<YOUR DOMAIN>".into(),
    "<YOUR KEY>".into(),
    sender,
    "<YOUR KANNON API HOST>".into(),
)
.await?;
```

> [!NOTE]
> Remember to add `https://` to the API host (e.g. `https://grpc.kannon.email:443`)!
> Also, the sender email should be part of your domain

To send mails, use the `send_mail` method:

```rust
let recipients = vec![Recipient {
    email: "test@mail.com".into(),
    fields: HashMap::from([("name".into(), "Test".into())]),
}];

kannon.send_email(
    recipients,
    "Hello from Kannon".into(), // Subject
    "<body>Hello from Kannon, {{ name }}!!</body>".into(), // Html Body
    vec![] // Attachments
)
.await?;
```

### Sending Templates
Similar to mails, you can send templates by indicating the template id instead of the mail body:

```rust
kannon.send_template(
    recipients,
    "Hello from Kannon".into(),
    "<template-id>".into(),
    vec![]
)
.await?;
```

### Credits
Developed (together with [@ludusrusso](https://github.com/ludusrusso)] during Open Source Saturday Milan.

[![Open Source Saturday](https://img.shields.io/badge/%E2%9D%A4%EF%B8%8F-open%20source%20saturday-F64060.svg)](https://www.meetup.com/it-IT/Open-Source-Saturday-Milano/)