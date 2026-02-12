use sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel, PrimaryKeyTrait,
};

use crate::error::{AppError, Result};

pub async fn get_model_by_id<M, T, ID>(db: &DatabaseConnection, id: ID) -> Result<Option<M::Model>>
where
    M: EntityTrait + Send + Sync,
    M::Model: IntoActiveModel<M::ActiveModel> + Send + Sync,
    M::ActiveModel: ActiveModelTrait + Send,
    T: Into<M::ActiveModel> + Send,
    ID: Into<<M::PrimaryKey as PrimaryKeyTrait>::ValueType> + Send,
{
    let model: Option<M::Model> = M::find_by_id(id.into())
        .one(db)
        .await
        .map_err(AppError::from)?;

    Ok(model)
}

pub async fn get_by_id<M, T, ID>(db: &DatabaseConnection, id: ID) -> Result<Option<T>>
where
    M: EntityTrait + Send + Sync,
    M::Model: IntoActiveModel<M::ActiveModel> + Send + Sync + Into<T>,
    M::ActiveModel: ActiveModelTrait + Send,
    T: Into<M::ActiveModel> + Send,
    ID: Into<<M::PrimaryKey as PrimaryKeyTrait>::ValueType> + Send,
{
    let model: Option<M::Model> = get_model_by_id::<M, T, ID>(db, id).await?;

    Ok(model.map(Into::into))
}

pub async fn create_model<M, T>(db: &DatabaseConnection, model: T) -> Result<M::Model>
where
    M: EntityTrait + Send + Sync,
    M::Model: IntoActiveModel<M::ActiveModel> + Send + Sync,
    M::ActiveModel: ActiveModelTrait + Send,
    T: Into<M::ActiveModel> + Send,
{
    let active_model = model.into();

    let result: M::Model = active_model.insert(db).await.map_err(AppError::from)?;

    Ok(result)
}

pub async fn create<M, T>(db: &DatabaseConnection, model: T) -> Result<T>
where
    M: EntityTrait + Send + Sync,
    M::Model: IntoActiveModel<M::ActiveModel> + Send + Sync + Into<T>,
    M::ActiveModel: ActiveModelTrait + Send,
    T: Into<M::ActiveModel> + Send,
{
    let result: M::Model = create_model::<M, T>(db, model).await?;

    Ok(result.into())
}

pub async fn update_model<M, T>(db: &DatabaseConnection, model: T) -> Result<M::Model>
where
    M: EntityTrait + Send + Sync,
    M::Model: IntoActiveModel<M::ActiveModel> + Send + Sync,
    M::ActiveModel: ActiveModelTrait + Send,
    T: Into<M::ActiveModel> + Send,
{
    let active_model = model.into();

    let result: M::Model = active_model.update(db).await.map_err(AppError::from)?;

    Ok(result)
}

pub async fn update<M, T>(db: &DatabaseConnection, model: T) -> Result<T>
where
    M: EntityTrait + Send + Sync,
    M::Model: IntoActiveModel<M::ActiveModel> + Send + Sync + Into<T>,
    M::ActiveModel: ActiveModelTrait + Send,
    T: Into<M::ActiveModel> + Send,
{
    let result: M::Model = update_model::<M, T>(db, model).await?;

    Ok(result.into())
}

pub async fn delete_model<M, T, ID>(db: &DatabaseConnection, id: ID) -> Result<M::Model>
where
    M: EntityTrait + Send + Sync,
    M::Model: IntoActiveModel<M::ActiveModel> + Send + Sync,
    M::ActiveModel: ActiveModelTrait + Send,
    T: Into<M::ActiveModel> + Send,
    ID: Into<<M::PrimaryKey as PrimaryKeyTrait>::ValueType> + Send,
{
    let res = M::delete_by_id(id)
        .exec_with_returning(db)
        .await
        .map_err(AppError::from)?;

    res.first().cloned().ok_or_else(|| {
        AppError::NotFound("Cannot delete entity because it does not exist".to_string())
    })
}

pub async fn delete<M, T, ID>(db: &DatabaseConnection, id: ID) -> Result<T>
where
    M: EntityTrait + Send + Sync,
    M::Model: IntoActiveModel<M::ActiveModel> + Send + Sync + Into<T>,
    M::ActiveModel: ActiveModelTrait + Send,
    T: Into<M::ActiveModel> + Send,
    ID: Into<<M::PrimaryKey as PrimaryKeyTrait>::ValueType> + Send,
{
    let result = delete_model::<M, T, ID>(db, id).await?;

    Ok(result.into())
}

pub async fn get_all_model<M, T>(db: &DatabaseConnection) -> Result<Vec<M::Model>>
where
    M: EntityTrait + Send + Sync,
    M::Model: Send + Sync,
    T: Send + Sync,
{
    let models = M::find().all(db).await.map_err(AppError::from)?;

    Ok(models)
}

pub async fn get_all<M, T>(db: &DatabaseConnection) -> Result<Vec<T>>
where
    M: EntityTrait + Send + Sync,
    M::Model: Send + Sync + Into<T>,
    T: Send + Sync,
{
    let models = get_all_model::<M, T>(db).await?;

    Ok(models.into_iter().map(Into::into).collect())
}
