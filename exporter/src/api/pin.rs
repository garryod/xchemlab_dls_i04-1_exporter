use async_graphql::{Context, InputObject, Object};
use derive_more::{Deref, DerefMut, From};
use models::bl_sample;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryTrait, Set};

#[derive(Debug, InputObject, Clone)]
pub struct PinInput {
    pub code: String,
}

pub trait FromInputAndPuckId {
    fn from_input_and_puck_id(input: PinInput, puck_id: u32) -> Self;
}

impl FromInputAndPuckId for bl_sample::ActiveModel {
    fn from_input_and_puck_id(input: PinInput, puck_id: u32) -> Self {
        Self {
            code: Set(Some(input.code)),
            container_id: Set(Some(puck_id)),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, From, Deref, DerefMut)]
pub struct Pin(bl_sample::Model);

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
        bl_sample::Entity::find()
            .apply_if(puck_id, |query, puck_id| {
                query.filter(bl_sample::Column::ContainerId.eq(puck_id))
            })
            .all(database)
            .await
            .map(|pins| pins.into_iter().map(Pin::from).collect())
            .map_err(async_graphql::Error::from)
    }
}
