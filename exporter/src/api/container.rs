use async_graphql::{Context, InputObject, Object};
use derive_more::{Deref, DerefMut, From};
use models::container;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryTrait, Set};

#[derive(Debug, InputObject, Clone)]
pub struct ContainerInput {
    pub code: String,
}

pub trait FromInputAndDewarId {
    fn from_input_and_dewar_id(input: ContainerInput, dewar_id: u32) -> Self;
}

impl FromInputAndDewarId for container::ActiveModel {
    fn from_input_and_dewar_id(input: ContainerInput, dewar_id: u32) -> Self {
        Self {
            dewar_id: Set(Some(dewar_id)),
            code: Set(Some(input.code)),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, From, Deref, DerefMut)]
pub struct Container(container::Model);

#[Object]
impl Container {
    async fn id(&self) -> &u32 {
        &self.container_id
    }

    async fn code(&self) -> &Option<String> {
        &self.code
    }
}

#[derive(Debug, Default)]
pub struct ContainerQuery;

#[Object]
impl ContainerQuery {
    async fn containers(
        &self,
        ctx: &Context<'_>,
        dewar_id: Option<u32>,
    ) -> async_graphql::Result<Vec<Container>> {
        let database = ctx.data::<DatabaseConnection>()?;
        container::Entity::find()
            .apply_if(dewar_id, |query, dewar_id| {
                query.filter(container::Column::DewarId.eq(dewar_id))
            })
            .all(database)
            .await
            .map(|containers| containers.into_iter().map(Container::from).collect())
            .map_err(async_graphql::Error::from)
    }
}
