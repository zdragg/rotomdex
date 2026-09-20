macro_rules! endpoint {
    (unnamed $type:ty; for $name:literal) => {
        crate::endpoint!(@inner $type; for $name);
    };
    ($type:ty; for $name:literal) => {
        crate::endpoint!(@inner $type; for $name);
    };
    (@inner $type:ty; for $name:literal) => {
        use crate::client::Client;
        use relative_path::RelativePath;
        use alloc::format;
        use crate::error::Error;

        pub async fn get_by_id(id: i64, client: &Client) -> Result<$type, Error> {
            let path = format!("/api/v2/{}/{id}", $name);
            client.get(RelativePath::new(&path)).await
        }

        pub async fn get_by_name(name: &str, client: &Client) -> Result<$type, Error> {
            let path = format!("/api/v2/{}/{name}", $name);
            client.get(RelativePath::new(&path)).await
        }
    };

    ($type:ty; for $name:literal; with $(($sub:ident, $sub_type:ty))+) => {

        crate::endpoint!($type; for $name);

        $(
            pub mod $sub {

                use super::Client;
                use relative_path::RelativePath;
                use alloc::{format, vec::Vec};
                use super::Error;

                pub async fn get_by_id(id: i64, client: &Client) -> Result<$sub_type, Error> {
                    let sub_path = format!("/api/v2/{}/{}/{}", $name, id, stringify!($sub));
                    client.get(RelativePath::new(&sub_path)).await
                }

                pub async fn get_by_name(name: &str, client: &Client) -> Result<$sub_type, Error> {
                    let sub_path = format!("/api/v2/{}/{}/{}", $name, name, stringify!($sub));
                    client.get(RelativePath::new(&sub_path)).await
                }
            }
        )+
    };
}

pub(crate) use endpoint;
