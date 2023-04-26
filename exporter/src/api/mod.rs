mod container;
mod dewar;
mod person;
mod proposals;
mod shipment;

use self::{
    container::ContainerQuery,
    dewar::DewarQuery,
    proposals::ProposalQuery,
    shipment::{ShipmentQuery, ShipmentSubscription},
    {person::PersonQuery, shipment::ShipmentMutation},
};
use async_graphql::{Enum, MergedObject, MergedSubscription, Schema};

pub type RootSchema = Schema<RootQuery, RootMutation, RootSubscription>;

#[derive(Debug, MergedObject, Default)]
pub struct RootQuery(
    ContainerQuery,
    DewarQuery,
    PersonQuery,
    ProposalQuery,
    ShipmentQuery,
);

#[derive(Debug, MergedObject, Default)]
pub struct RootMutation(ShipmentMutation);

#[derive(Debug, Enum, PartialEq, Eq, Clone, Copy)]
pub enum MutationType {
    Created,
    Deleted,
}

#[derive(Debug, MergedSubscription, Default)]
pub struct RootSubscription(ShipmentSubscription);
