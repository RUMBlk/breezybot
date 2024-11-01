macro_rules! insert {
    ($($table:tt)::+, $($column:ident),* $(,)?) => {
        $($table)::+::Entity::insert(
            $($table)::+::ActiveModel {
                $($column: sea_orm::Set($value),)*
                ..Default::default()
            }
        )
    };

    ($($table:tt)::+, $($column:ident: $value:expr $(,)? )*) => {
        $($table)::+::Entity::insert(
            $($table)::+::ActiveModel {
                $($column: sea_orm::Set($value),)*
                ..Default::default()
            }
        )
    };
}

macro_rules! update {
    ($db: expr, $model: expr, $($column:ident),* $(,)?) => {
        {
            let mut active = $model.into_active_model();
            $(active.$column = sea_orm::Set($column);)*
            active.update($db)
        }
    };
    ($db: expr, $model: expr, $($column:ident: $value:expr $(,)? )*) => {
        {
            let mut active = $model.into_active_model();
            $(active.$column = sea_orm::Set($value);)*
            active.update($db)
        }
    };
}
pub(crate) use update;
