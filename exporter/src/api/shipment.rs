use super::{
    dewar::{Dewar, DewarInput, FromInputAndShippingId},
    proposals::Proposal,
    puck::FromInputAndDewarId,
};
use crate::broker::EventBroker;
use async_graphql::{
    futures_util::{stream::FuturesOrdered, Stream, StreamExt},
    Context, Object, Subscription,
};
use derive_more::{Deref, DerefMut, From};
use models::{container, dewar, proposal, shipping};
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryTrait, Set};

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

    async fn dewars(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Dewar>> {
        let database = ctx.data::<DatabaseConnection>()?;
        Ok(dewar::Entity::find()
            .filter(dewar::Column::ShippingId.eq(self.shipping_id))
            .all(database)
            .await?
            .into_iter()
            .map(Dewar::from)
            .collect())
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

#[derive(Debug, Default)]
pub struct ShipmentMutation;

#[derive(Debug, Clone)]
pub enum ShipmentEvent {
    Created(Shipment),
}

static SHIPMENT_EVENT_BROKER: EventBroker<ShipmentEvent> = EventBroker::<ShipmentEvent>::new();

#[Object]
impl ShipmentMutation {
    async fn create_shipment(
        &self,
        ctx: &Context<'_>,
        proposal_id: u32,
        dewars: Vec<DewarInput>,
    ) -> async_graphql::Result<Shipment> {
        let database = ctx.data::<DatabaseConnection>()?;

        let shipping_model = shipping::ActiveModel {
            proposal_id: Set(proposal_id),
            shipping_name: Set(Some("XChemLab Shipment".to_string())),
            comments: Set(Some(
                "Automatically generated by XChemLab DLS i04-1 Exporter".to_string(),
            )),
            ..Default::default()
        };
        let shipping_insert = shipping::Entity::insert(shipping_model)
            .exec(database)
            .await?;

        dewars
            .into_iter()
            .map(|dewar| async {
                let dewar_insert =
                    dewar::Entity::insert(dewar::ActiveModel::from_input_and_shipping_id(
                        dewar.clone(),
                        shipping_insert.last_insert_id,
                    ))
                    .exec(database)
                    .await?;

                let puck_inserts = dewar
                    .pucks
                    .into_iter()
                    .map(|puck| async {
                        container::Entity::insert(container::ActiveModel::from_input_and_dewar_id(
                            puck,
                            dewar_insert.last_insert_id,
                        ))
                        .exec(database)
                        .await
                    })
                    .collect::<FuturesOrdered<_>>()
                    .collect::<Vec<_>>()
                    .await
                    .into_iter()
                    .collect::<Result<Vec<_>, DbErr>>()?;

                Ok((dewar_insert, puck_inserts))
            })
            .collect::<FuturesOrdered<_>>()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, DbErr>>()?;

        let created_shipping = shipping::Entity::find_by_id(shipping_insert.last_insert_id)
            .one(database)
            .await?
            .map(Shipment::from)
            .ok_or(async_graphql::Error::new(&format!(
                "Inserted model at {} but could not retrieve copy",
                shipping_insert.last_insert_id
            )))?;

        SHIPMENT_EVENT_BROKER.publish(ShipmentEvent::Created(created_shipping.clone()));

        Ok(created_shipping)
    }
}

#[derive(Debug, Default)]
pub struct ShipmentSubscription;

#[Subscription]
impl ShipmentSubscription {
    async fn shipment_created(&self) -> impl Stream<Item = Shipment> {
        SHIPMENT_EVENT_BROKER
            .subscribe()
            .filter_map(move |event| async move {
                if let Ok(ShipmentEvent::Created(shipment)) = event {
                    Some(shipment)
                } else {
                    None
                }
            })
    }
}
