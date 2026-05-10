mod select;
mod insert;
mod update;
mod delete;

pub use select::Select;
pub use insert::Insert;
pub use update::Update;
pub use delete::{Delete, Filtered, Unfiltered};

use crate::{
    ast::FromSource,
    convert::FromBatch,
    errors::Error,
    model::Model,
    view::View,
};


/// Provides query and mutation entry points for any type implementing [`Model`].
pub trait EntityOps: Model + FromBatch + Sized {
    /// Returns a new [`Select`] for this model.
    fn find() -> Select<Self> {
        Select::new(FromSource::table(Self::table_name()))
    }

    /// Returns an [`Insert`] for a single model instance.
    fn insert(model: Self) -> Result<Insert<Self>, Error> {
        Insert::one(model)
    }

    /// Returns an [`Insert`] for a batch of model instances.
    fn insert_batch(models: Vec<Self>) -> Result<Insert<Self>, Error> {
        Insert::many(models)
    }

    /// Returns a new [`Update`] for this model.
    fn update() -> Update<Self> {
        Update::new()
    }

    /// Returns a new unfiltered [`Delete`] for this model.
    fn delete() -> Delete<Self, Unfiltered> {
        Delete::new()
    }

    /// Returns a [`Delete`] that targets all rows in the table.
    fn delete_all() -> Delete<Self, Filtered> {
        Delete::all()
    }
}

impl<M: Model + FromBatch> EntityOps for M {}

/// Provides a `find` entry point for any type implementing [`View`].
pub trait ViewOps: View + FromBatch + Sized {
    /// Returns a new [`Select`] for this view.
    fn find() -> Select<Self> {
        Select::new(FromSource::table(Self::view_name()))
    }
}

impl<V: View + FromBatch> ViewOps for V {}
