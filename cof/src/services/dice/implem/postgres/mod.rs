use anyhow::{Context, anyhow};
use sqlx::{PgPool, prelude::*};
use std::sync::Arc;
use tonic::async_trait;

use crate::model::dice::{Dice, RolledDice, RolledDiceSet};
use crate::services::dice::{DiceHistorySaver, Error, RollId};

#[derive(Debug)]
pub struct PostgresRepo {
    pool: Arc<PgPool>,
}

impl PostgresRepo {
    /// Create a Postgres Resporitory that implements the [`DiceHistorySaver`]
    ///
    /// # Errors
    ///
    /// The Error cans be a `sqlx::MigrateError` if the migration fails.
    pub async fn new(pg_pool: PgPool) -> Result<Self, Error> {
        sqlx::migrate!("./src/services/dice/implem/postgres/migrations")
            .run(&pg_pool)
            .await
            .context("Failed to run the Postgres migration when starting the repository")?;

        Ok(Self {
            pool: Arc::new(pg_pool),
        })
    }
}

#[derive(FromRow)]
struct RolledDiceDbEntry {
    dice: String,
    result: i64,
}

impl TryFrom<RolledDiceDbEntry> for RolledDice {
    type Error = anyhow::Error;

    fn try_from(value: RolledDiceDbEntry) -> Result<Self, Self::Error> {
        let dice = Dice::try_from(value.dice.as_str())
            .context("cannot decode the value of the Dice stored in the database")?;
        let result = u32::try_from(value.result)
            .context("cannot decode the value of the result of the roll stored in the database")?;

        Ok(Self::new(dice, result))
    }
}

#[async_trait]
impl DiceHistorySaver for PostgresRepo {
    async fn save_roll(&self, id: &RollId, rolled_dice_set: &RolledDiceSet) -> Result<(), Error> {
        let roll_ids = rolled_dice_set
            .iter()
            .map(|_| *id.as_ref())
            .collect::<Vec<_>>();
        let dices = rolled_dice_set
            .iter()
            .map(|rds| rds.dice().to_string())
            .collect::<Vec<_>>();
        let results = rolled_dice_set
            .iter()
            .map(|rds| i64::from(rds.result()))
            .collect::<Vec<_>>();

        let rows_affected = sqlx::query!(
            r#"INSERT INTO dice_rolls (roll_id, dice, result) SELECT * FROM UNNEST(
                $1::uuid[],
                $2::VARCHAR(5)[],
                $3::BIGINT[]
            )"#,
            &roll_ids,
            &dices,
            &results,
        )
        .execute(&*self.pool)
        .await
        .context("error inserting entries into the database")?
        .rows_affected();

        if rows_affected != (roll_ids.len() as u64) {
            return Err(Error::Underlying(anyhow!(
                "Postgres transaction error detected not all rolls have been persisted",
            )));
        }

        Ok(())
    }

    async fn get_dice_roll(&self, id: &RollId) -> Result<RolledDiceSet, Error> {
        let roll_id = *id.as_ref();

        let rolled_dices = sqlx::query_as!(
            RolledDiceDbEntry,
            r#"SELECT dice, result FROM dice_rolls WHERE roll_id = $1"#,
            roll_id
        )
        .fetch_all(&*self.pool)
        .await
        .context("error reading dice rolls from postgres database")?;

        let rolled_dices = rolled_dices
            .into_iter()
            .map(RolledDice::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(RolledDiceSet::new(rolled_dices.into_iter()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use sqlx::PgPool;
    use testcontainers::ContainerAsync;
    use testcontainers_modules::postgres::Postgres;
    use testcontainers_modules::testcontainers::runners::AsyncRunner;
    use uuid::Uuid;

    use crate::model::dice::{Dice, DiceSet};

    async fn make_postgres_pool() -> (ContainerAsync<Postgres>, PgPool) {
        // startup the module
        let node: testcontainers::ContainerAsync<Postgres> = Postgres::default()
            .start()
            .await
            .expect("Cannot start Postgres Test Container");

        let pg_pool = sqlx::PgPool::connect(&format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            node.get_host_port_ipv4(5432)
                .await
                .expect("Cannot fetch TestContainer port for Postgres")
        ))
        .await
        .expect("Cannot Create PgPool");

        (node, pg_pool)
    }

    #[tokio::test]
    async fn can_save_and_get_dice_rolls() {
        let (_node, pg_pool) = make_postgres_pool().await;
        let sut = PostgresRepo::new(pg_pool)
            .await
            .unwrap_or_else(|e| panic!("Cannot instanciate Postgres Repo: {e}"));

        let id = RollId::from(Uuid::now_v7());
        let rolled_dice_set = DiceSet::new([Dice::D100, Dice::D10].iter().copied())
            .roll()
            .unwrap();

        let save_result = sut.save_roll(&id, &rolled_dice_set).await;
        assert!(save_result.is_ok());

        let get_rolled_dice_result = sut.get_dice_roll(&id).await;
        assert!(get_rolled_dice_result.is_ok());
        assert_eq!(get_rolled_dice_result.unwrap(), rolled_dice_set);
    }
}
