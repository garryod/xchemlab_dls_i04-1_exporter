mod person;
mod proposals;

use self::person::PersonQuery;
use self::proposals::ProposalQuery;
use async_graphql::{EmptyMutation, EmptySubscription, Enum, MergedObject, Schema};

pub type RootSchema = Schema<RootQuery, RootMutation, RootSubscription>;

#[derive(Debug, MergedObject, Default)]
pub struct RootQuery(PersonQuery, ProposalQuery);

pub type RootMutation = EmptyMutation;

#[derive(Debug, Enum, PartialEq, Eq, Clone, Copy)]
pub enum MutationType {
    Created,
    Deleted,
}

pub type RootSubscription = EmptySubscription;
