use async_graphql::{Context, InputObject, Object};
use derive_more::{Deref, DerefMut, From};
use models::dewar;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryTrait, Set};

#[derive(Debug, InputObject)]
pub struct DewarInput {
    code: String,
}

pub trait FromInputAndShippingId {
    fn from_input_and_shipping_id(input: DewarInput, shipping_id: u32) -> Self;
}

impl FromInputAndShippingId for dewar::ActiveModel {
    fn from_input_and_shipping_id(input: DewarInput, shipping_id: u32) -> Self {
        Self {
            shipping_id: Set(Some(shipping_id)),
            code: Set(Some(input.code)),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, From, Deref, DerefMut)]
pub struct Dewar(dewar::Model);

#[Object]
impl Dewar {
    async fn id(&self) -> &u32 {
        &self.dewar_id
    }

    async fn code(&self) -> &Option<String> {
        &self.code
    }
}

#[derive(Debug, Default)]
pub struct DewarQuery;

#[Object]
impl DewarQuery {
    async fn dewars(
        &self,
        ctx: &Context<'_>,
        shipment_id: Option<u32>,
    ) -> async_graphql::Result<Vec<Dewar>> {
        let database = ctx.data::<DatabaseConnection>()?;
        dewar::Entity::find()
            .apply_if(shipment_id, |query, shipment_id| {
                query.filter(dewar::Column::ShippingId.eq(shipment_id))
            })
            .all(database)
            .await
            .map(|dewars| dewars.into_iter().map(Dewar::from).collect())
            .map_err(async_graphql::Error::from)
    }
}
