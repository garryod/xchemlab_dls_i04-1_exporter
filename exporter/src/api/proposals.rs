use super::person::Person;
use async_graphql::{Context, Enum, Object};
use derive_more::{Deref, DerefMut, From};
use models::{person, proposal, sea_orm_active_enums};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryTrait};

#[derive(Debug, Enum, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Open,
    Closed,
    Cancelled,
}

impl From<sea_orm_active_enums::State> for State {
    fn from(value: sea_orm_active_enums::State) -> Self {
        match value {
            sea_orm_active_enums::State::Open => Self::Open,
            sea_orm_active_enums::State::Closed => Self::Closed,
            sea_orm_active_enums::State::Cancelled => Self::Cancelled,
        }
    }
}

impl From<State> for sea_orm_active_enums::State {
    fn from(value: State) -> Self {
        match value {
            State::Open => Self::Open,
            State::Closed => Self::Closed,
            State::Cancelled => Self::Cancelled,
        }
    }
}

#[derive(Debug, Clone, From, Deref, DerefMut)]
pub struct Proposal(proposal::Model);

#[Object]
impl Proposal {
    async fn id(&self) -> &u32 {
        &self.proposal_id
    }

    async fn person(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<Person>> {
        let database = ctx.data::<DatabaseConnection>()?;
        Ok(person::Entity::find_by_id(self.person_id)
            .one(database)
            .await?
            .map(Person::from))
    }

    async fn title(&self) -> &Option<String> {
        &self.title
    }

    async fn code(&self) -> &Option<String> {
        &self.proposal_code
    }

    async fn number(&self) -> &Option<String> {
        &self.proposal_number
    }

    async fn state(&self) -> Option<State> {
        self.state.map(State::from)
    }
}

#[derive(Debug, Default)]
pub struct ProposalQuery;

#[Object]
impl ProposalQuery {
    async fn proposals(
        &self,
        ctx: &Context<'_>,
        id: Option<u32>,
    ) -> async_graphql::Result<Vec<Proposal>> {
        let database = ctx.data::<DatabaseConnection>()?;
        proposal::Entity::find()
            .apply_if(id, |query, id| {
                query.filter(proposal::Column::ProposalId.eq(id))
            })
            .all(database)
            .await
            .map(|proposals| proposals.into_iter().map(Proposal::from).collect())
            .map_err(async_graphql::Error::from)
    }
}
