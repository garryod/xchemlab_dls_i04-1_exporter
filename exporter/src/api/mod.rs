mod person;
mod proposals;
mod shipment;

use self::{
    proposals::ProposalQuery,
    shipment::ShipmentQuery,
    {person::PersonQuery, shipment::ShipmentMutation},
};
use async_graphql::{EmptySubscription, Enum, MergedObject, Schema};

pub type RootSchema = Schema<RootQuery, RootMutation, RootSubscription>;

#[derive(Debug, MergedObject, Default)]
pub struct RootQuery(PersonQuery, ProposalQuery, ShipmentQuery);

#[derive(Debug, MergedObject, Default)]
pub struct RootMutation(ShipmentMutation);

#[derive(Debug, Enum, PartialEq, Eq, Clone, Copy)]
pub enum MutationType {
    Created,
    Deleted,
}

pub type RootSubscription = EmptySubscription;
