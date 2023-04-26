use async_graphql::{Context, InputObject, Object};
use derive_more::{Deref, DerefMut, From};
use models::container;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryTrait, Set};

#[derive(Debug, InputObject, Clone)]
pub struct PuckInput {
    pub code: String,
}

pub trait FromInputAndDewarId {
    fn from_input_and_dewar_id(input: PuckInput, dewar_id: u32) -> Self;
}

impl FromInputAndDewarId for container::ActiveModel {
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
pub struct Puck(container::Model);

#[Object]
impl Puck {
    async fn id(&self) -> &u32 {
        &self.container_id
    }

    async fn code(&self) -> &Option<String> {
        &self.code
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
        container::Entity::find()
            .apply_if(dewar_id, |query, dewar_id| {
                query.filter(container::Column::DewarId.eq(dewar_id))
            })
            .all(database)
            .await
            .map(|pucks| pucks.into_iter().map(Puck::from).collect())
            .map_err(async_graphql::Error::from)
    }
}
