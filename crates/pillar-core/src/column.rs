use crate::{value::Value, condition::ConditionExpression};


#[derive(Debug, Clone, PartialEq)]
pub enum ColumnType {
    Boolean,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
    String,
    Binary,
    List(Box<ColumnType>),
    Map(Box<ColumnType>, Box<ColumnType>),
    #[cfg(feature = "chrono")]
    Date,
    #[cfg(feature = "chrono")]
    Time,
    #[cfg(feature = "chrono")]
    DateTime,
    #[cfg(feature = "uuid")]
    Uuid,
}

#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: &'static str,
    pub column_type: ColumnType,
    pub nullable: bool,
    pub primary_key: bool,
    pub unique: bool,
}


pub trait IntoColumnRef {
    fn into_column_ref(self) -> String;
}

impl IntoColumnRef for String {
    fn into_column_ref(self) -> String {
        self
    }
}

impl IntoColumnRef for &String {
    fn into_column_ref(self) -> String {
        self.clone()
    }
}

impl IntoColumnRef for &str {
    fn into_column_ref(self) -> String {
        self.to_string()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TypedColumn<T> {
    name: &'static str,
    _marker: std::marker::PhantomData<T>,
}

impl<T> TypedColumn<T> {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl<T> IntoColumnRef for TypedColumn<T> {
    fn into_column_ref(self) -> String {
        self.name.to_string()
    }
}

impl<T> TypedColumn<T>
where
    T: Into<Value>,
{
    pub fn eq<V: Into<Value>>(self, value: V) -> ConditionExpression {
        ConditionExpression::eq(self.into_column_ref(), value.into())
    }

    pub fn ne<V: Into<Value>>(self, value: V) -> ConditionExpression {
        ConditionExpression::ne(self.into_column_ref(), value.into())
    }

    pub fn gt<V: Into<Value>>(self, value: V) -> ConditionExpression {
        ConditionExpression::gt(self.into_column_ref(), value.into())
    }

    pub fn gte<V: Into<Value>>(self, value: V) -> ConditionExpression {
        ConditionExpression::gte(self.into_column_ref(), value.into())
    }

    pub fn lt<V: Into<Value>>(self, value: V) -> ConditionExpression {
        ConditionExpression::lt(self.into_column_ref(), value.into())
    }

    pub fn lte<V: Into<Value>>(self, value: V) -> ConditionExpression {
        ConditionExpression::lte(self.into_column_ref(), value.into())
    }

    pub fn in_list<I, V>(self, values: I) -> ConditionExpression
    where
        I: IntoIterator<Item = V>,
        V: Into<Value>,
    {
        ConditionExpression::in_list(self.into_column_ref(), values.into_iter().map(Into::into).collect())
    }

    pub fn is_not_in<I, V>(self, values: I) -> ConditionExpression
    where
        I: IntoIterator<Item = V>,
        V: Into<Value>,
    {
        ConditionExpression::is_not_in(self.into_column_ref(), values.into_iter().map(Into::into).collect())
    }

    pub fn is_null(self) -> ConditionExpression {
        ConditionExpression::is_null(self.into_column_ref())
    }

    pub fn is_not_null(self) -> ConditionExpression {
        ConditionExpression::is_not_null(self.into_column_ref())
    }

    pub fn between<V: Into<Value>>(self, low: V, high: V) -> ConditionExpression {
        ConditionExpression::between(self.into_column_ref(), low.into(), high.into())
    }

    pub fn not_between<V: Into<Value>>(self, low: V, high: V) -> ConditionExpression {
        ConditionExpression::not_between(self.into_column_ref(), low.into(), high.into())
    }
}

impl<T> TypedColumn<T>
where
    T: AsRef<str>,
{
    pub fn like(self, pattern: impl Into<String>) -> ConditionExpression {
        ConditionExpression::like(self.into_column_ref(), pattern.into())
    }

    pub fn not_like(self, pattern: impl Into<String>) -> ConditionExpression {
        ConditionExpression::not_like(self.into_column_ref(), pattern.into())
    }
}
