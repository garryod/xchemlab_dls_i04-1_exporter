mod dewar;
mod person;
mod pin;
mod proposal;
mod puck;
mod shipment;

use self::{
    dewar::DewarQuery,
    pin::PinQuery,
    proposal::ProposalQuery,
    puck::PuckQuery,
    shipment::{ShipmentQuery, ShipmentSubscription},
    {person::PersonQuery, shipment::ShipmentMutation},
};
use async_graphql::{Enum, MergedObject, MergedSubscription, Schema};

pub type RootSchema = Schema<RootQuery, RootMutation, RootSubscription>;

#[derive(Debug, MergedObject, Default)]
pub struct RootQuery(
    DewarQuery,
    PersonQuery,
    PinQuery,
    ProposalQuery,
    PuckQuery,
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
