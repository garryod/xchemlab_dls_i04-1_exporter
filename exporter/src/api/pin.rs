use async_graphql::{Context, InputObject, Object};
use derive_more::{Deref, DerefMut, From};
use models::bl_sample::{ActiveModel, Column, Entity, Model};
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, InsertResult, QueryFilter, QueryTrait, Set,
};

#[derive(Debug, InputObject, Clone)]
pub struct PinInput {
    pub code: String,
}

impl PinInput {
    pub async fn insert_as_child(
        self,
        puck_id: u32,
        database: &DatabaseConnection,
    ) -> Result<InsertResult<ActiveModel>, DbErr> {
        Entity::insert(ActiveModel {
            container_id: Set(Some(puck_id)),
            code: Set(Some(self.code)),
            ..Default::default()
        })
        .exec(database)
        .await
    }
}

#[derive(Debug, Clone, From, Deref, DerefMut)]
pub struct Pin(Model);

#[Object]
impl Pin {
    async fn id(&self) -> &u32 {
        &self.bl_sample_id
    }

    async fn code(&self) -> &Option<String> {
        &self.code
    }
}

#[derive(Debug, Default)]
pub struct PinQuery;

#[Object]
impl PinQuery {
    async fn pins(
        &self,
        ctx: &Context<'_>,
        puck_id: Option<u32>,
    ) -> async_graphql::Result<Vec<Pin>> {
        let database = ctx.data::<DatabaseConnection>()?;
        Entity::find()
            .apply_if(puck_id, |query, puck_id| {
                query.filter(Column::ContainerId.eq(puck_id))
            })
            .all(database)
            .await
            .map(|pins| pins.into_iter().map(Pin::from).collect())
            .map_err(async_graphql::Error::from)
    }
}
