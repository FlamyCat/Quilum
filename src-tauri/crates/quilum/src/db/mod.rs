use surrealdb::types::RecordId;

pub(crate) struct Record<T> {
    pub(crate) id: RecordId,
    pub(crate) data: T,
}
