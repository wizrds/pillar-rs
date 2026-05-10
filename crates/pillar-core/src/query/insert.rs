use crate::{
    ast::{
        InsertStatement,
        Statement,
        TableRef,
    },
    database::Database,
    errors::Error,
    model::Model,
};


/// A builder for an `INSERT` statement targeting a [`Model`] table.
#[derive(Debug, Clone)]
pub struct Insert<M: Model> {
    statement: InsertStatement,
    _marker: std::marker::PhantomData<M>,
}

impl<M: Model> Insert<M> {
    /// Creates an insert for a batch of models.
    pub fn many(models: Vec<M>) -> Result<Self, Error> {
        if models.is_empty() {
            return Err(Error::invalid_query("Cannot insert empty batch"));
        }

        Ok(Self {
            statement: InsertStatement::new(TableRef::new(M::table_name()))
                .columns(M::columns().iter().map(|col| col.name.to_string()))
                .values(models.iter().map(|m| m.to_row())),
            _marker: std::marker::PhantomData,
        })
    }

    /// Creates an insert for a single model.
    pub fn one(model: M) -> Result<Self, Error> {
        Self::many(vec![model])
    }

    /// Converts this builder into a [`Statement`].
    pub fn into_statement(self) -> Statement {
        Statement::Insert(self.statement)
    }

    /// Executes the insert and returns the number of rows affected.
    pub async fn execute<D: Database>(self, database: &D) -> Result<usize, Error> {
        Ok(
            database
                .execute(&self.into_statement())
                .await?
                .rows_affected
        )
    }
}
