use super::pin::{Pin, PinInput};
use async_graphql::{
    futures_util::{stream::FuturesOrdered, StreamExt},
    Context, InputObject, Object,
};
use derive_more::{Deref, DerefMut, From};
use models::{
    bl_sample,
    container::{ActiveModel, Column, Entity, Model},
};
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, InsertResult, QueryFilter, QueryTrait, Set,
};

#[derive(Debug, InputObject, Clone)]
pub struct PuckInput {
    pub code: String,
    pub pins: Vec<PinInput>,
}

impl PuckInput {
    pub async fn insert_as_child_recursive(
        self,
        dewar_id: u32,
        database: &DatabaseConnection,
    ) -> Result<
        (
            InsertResult<ActiveModel>,
            Vec<InsertResult<bl_sample::ActiveModel>>,
        ),
        DbErr,
    > {
        let insert = Entity::insert(ActiveModel {
            dewar_id: Set(Some(dewar_id)),
            code: Set(Some(self.code)),
            ..Default::default()
        })
        .exec(database)
        .await?;

        let pin_inserts = self
            .pins
            .into_iter()
            .map(|pin| pin.insert_as_child(insert.last_insert_id, database))
            .collect::<FuturesOrdered<_>>()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, DbErr>>()?;

        Ok((insert, pin_inserts))
    }
}

pub trait FromInputAndDewarId {
    fn from_input_and_dewar_id(input: PuckInput, dewar_id: u32) -> Self;
}

impl FromInputAndDewarId for ActiveModel {
    fn from_input_and_dewar_id(input: PuckInput, dewar_id: u32) -> Self {
        Self {
            dewar_id: Set(Some(dewar_id)),
            code: Set(Some(input.code)),
            container_type: Set(Some("Puck".to_string())),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, From, Deref, DerefMut)]
pub struct Puck(Model);

#[Object]
impl Puck {
    async fn id(&self) -> &u32 {
        &self.container_id
    }

    async fn code(&self) -> &Option<String> {
        &self.code
    }

    async fn pins(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Pin>> {
        let database = ctx.data::<DatabaseConnection>()?;
        Ok(bl_sample::Entity::find()
            .filter(bl_sample::Column::ContainerId.eq(self.container_id))
            .all(database)
            .await?
            .into_iter()
            .map(Pin::from)
            .collect())
    }
}

#[derive(Debug, Default)]
pub struct PuckQuery;

#[Object]
impl PuckQuery {
    async fn pucks(
        &self,
        ctx: &Context<'_>,
        dewar_id: Option<u32>,
    ) -> async_graphql::Result<Vec<Puck>> {
        let database = ctx.data::<DatabaseConnection>()?;
        Entity::find()
            .apply_if(dewar_id, |query, dewar_id| {
                query.filter(Column::DewarId.eq(dewar_id))
            })
            .all(database)
            .await
            .map(|pucks| pucks.into_iter().map(Puck::from).collect())
            .map_err(async_graphql::Error::from)
    }
}
