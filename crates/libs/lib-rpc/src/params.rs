use modql::filter::ListOptions;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_with::{OneOrMany, serde_as};


/// For all APIs that want to create something.
#[derive(Deserialize)]
pub struct ParamsForCreate<D> {
    pub data: D,
}

/// For all APIs that want to update something.
#[derive(Deserialize)]
pub struct ParamsForUpdate<D> {
    pub id: i64,
    pub data: D,
}

/// For all APIs that just parse the Id, e.g. delete or get.
#[derive(Deserialize)]
pub struct ParamsIded {
    pub id: i64,
}

/// For all APIs that need lists
#[serde_as]
#[derive(Deserialize)]
pub struct ParamsList<F>
where
    F: DeserializeOwned,
{
    // F needs to work for single filters and an array of filters
    #[serde_as(deserialize_as = "Option<OneOrMany<_>>")]
    pub filters: Option<Vec<F>>,
    pub list_options: Option<ListOptions>,
}
