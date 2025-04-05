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