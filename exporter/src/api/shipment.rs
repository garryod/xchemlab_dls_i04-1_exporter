use super::proposals::Proposal;
use async_graphql::{Context, Object};
use derive_more::{Deref, DerefMut, From};
use models::{proposal, shipping};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryTrait};

#[derive(Debug, Clone, From, Deref, DerefMut)]
pub struct Shipment(shipping::Model);

#[Object]
impl Shipment {
    async fn id(&self) -> &u32 {
        &self.shipping_id
    }

    async fn proposal(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<Proposal>> {
        let database = ctx.data::<DatabaseConnection>()?;
        Ok(proposal::Entity::find_by_id(self.proposal_id)
            .one(database)
            .await?
            .map(Proposal::from))
    }

    async fn name(&self) -> &Option<String> {
        &self.shipping_name
    }

    async fn comments(&self) -> &Option<String> {
        &self.comments
    }
}

#[derive(Debug, Default)]
pub struct ShipmentQuery;

#[Object]
impl ShipmentQuery {
    async fn shipments(
        &self,
        ctx: &Context<'_>,
        proposal_id: Option<u32>,
    ) -> async_graphql::Result<Vec<Shipment>> {
        let database = ctx.data_unchecked::<DatabaseConnection>();
        shipping::Entity::find()
            .apply_if(proposal_id, |query, proposal_id| {
                query.filter(shipping::Column::ProposalId.eq(proposal_id))
            })
            .all(database)
            .await
            .map(|shippings| shippings.into_iter().map(Shipment::from).collect())
            .map_err(async_graphql::Error::from)
    }
}
