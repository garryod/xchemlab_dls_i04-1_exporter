use super::puck::{Puck, PuckInput};
use async_graphql::{
    futures_util::{stream::FuturesOrdered, StreamExt},
    Context, InputObject, Object,
};
use derive_more::{Deref, DerefMut, From};
use models::{
    bl_sample, container,
    dewar::{ActiveModel, Column, Entity, Model},
};
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, InsertResult, QueryFilter, QueryTrait, Set,
};

#[derive(Debug, InputObject, Clone)]
pub struct DewarInput {
    pub code: String,
    pub pucks: Vec<PuckInput>,
}

impl DewarInput {
    pub async fn insert_as_child_recursive(
        self,
        shipment_id: u32,
        database: &DatabaseConnection,
    ) -> Result<
        (
            InsertResult<ActiveModel>,
            Vec<(
                InsertResult<container::ActiveModel>,
                Vec<InsertResult<bl_sample::ActiveModel>>,
            )>,
        ),
        DbErr,
    > {
        let insert = Entity::insert(ActiveModel {
            shipping_id: Set(Some(shipment_id)),
            code: Set(Some(self.code)),
            ..Default::default()
        })
        .exec(database)
        .await?;

        let puck_inserts = self
            .pucks
            .into_iter()
            .map(|puck| puck.insert_as_child_recursive(insert.last_insert_id, database))
            .collect::<FuturesOrdered<_>>()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, DbErr>>()?;

        Ok((insert, puck_inserts))
    }
}

pub trait FromInputAndShippingId {
    fn from_input_and_shipping_id(input: DewarInput, shipping_id: u32) -> Self;
}

impl FromInputAndShippingId for ActiveModel {
    fn from_input_and_shipping_id(input: DewarInput, shipping_id: u32) -> Self {
        Self {
            shipping_id: Set(Some(shipping_id)),
            code: Set(Some(input.code)),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, From, Deref, DerefMut)]
pub struct Dewar(Model);

#[Object]
impl Dewar {
    async fn id(&self) -> &u32 {
        &self.dewar_id
    }

    async fn code(&self) -> &Option<String> {
        &self.code
    }

    async fn pucks(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Puck>> {
        let database = ctx.data::<DatabaseConnection>()?;
        Ok(container::Entity::find()
            .filter(container::Column::DewarId.eq(self.dewar_id))
            .all(database)
            .await?
            .into_iter()
            .map(Puck::from)
            .collect())
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
        Entity::find()
            .apply_if(shipment_id, |query, shipment_id| {
                query.filter(Column::ShippingId.eq(shipment_id))
            })
            .all(database)
            .await
            .map(|dewars| dewars.into_iter().map(Dewar::from).collect())
            .map_err(async_graphql::Error::from)
    }
}
