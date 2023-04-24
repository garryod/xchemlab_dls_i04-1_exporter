mod person;

use async_graphql::{EmptyMutation, EmptySubscription, Enum, MergedObject, Schema};
use person::PersonQuery;

pub type RootSchema = Schema<RootQuery, RootMutation, RootSubscription>;

#[derive(Debug, MergedObject, Default)]
pub struct RootQuery(PersonQuery);

pub type RootMutation = EmptyMutation;

#[derive(Debug, Enum, PartialEq, Eq, Clone, Copy)]
pub enum MutationType {
    Created,
    Deleted,
}

pub type RootSubscription = EmptySubscription;
