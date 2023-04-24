use async_graphql::{Context, Object};
use derive_more::{Deref, DerefMut, From};
use models::person;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryTrait};

#[derive(Debug, Clone, From, Deref, DerefMut)]
pub struct Person(person::Model);

#[Object]
impl Person {
    async fn id(&self) -> &u32 {
        &self.person_id
    }

    async fn name(&self) -> String {
        match (&self.given_name, &self.family_name, &self.title) {
            (None, None, None) => "Unknown".to_string(),
            (None, None, Some(title)) => format!("{} Unknown", title),
            (None, Some(family_name), None) => family_name.clone(),
            (None, Some(family_name), Some(title)) => format!("{} {}", title, family_name),
            (Some(given_name), None, None) => given_name.clone(),
            (Some(given_name), None, Some(_)) => given_name.clone(),
            (Some(given_name), Some(family_name), None) => {
                format!("{} {}", given_name, family_name)
            }
            (Some(given_name), Some(family_name), Some(title)) => {
                format!("{} {} {}", title, given_name, family_name)
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct PersonQuery;

#[Object]
impl PersonQuery {
    async fn people(
        &self,
        ctx: &Context<'_>,
        id: Option<u32>,
    ) -> async_graphql::Result<Vec<Person>> {
        let database = ctx.data::<DatabaseConnection>()?;
        person::Entity::find()
            .apply_if(id, |query, id| {
                query.filter(person::Column::PersonId.eq(id))
            })
            .all(database)
            .await
            .map(|people| people.into_iter().map(Person::from).collect())
            .map_err(async_graphql::Error::from)
    }
}
