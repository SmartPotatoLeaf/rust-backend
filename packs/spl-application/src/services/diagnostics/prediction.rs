use crate::dtos::diagnostics::CreatePredictionDto;
use crate::mappers::diagnostics::prediction::CreatePredictionContext;
use crate::services::access_control::AccessControlService;

use crate::dtos::diagnostics::FilterPredictionDto;
use spl_domain::entities::diagnostics::{Prediction, PredictionMark};
use spl_domain::entities::image::Image;
use spl_domain::entities::user::User;
use spl_domain::ports::integrations::{BlobStorageClient, ModelPredictionClient};
use spl_domain::ports::repositories::diagnostics::{
    LabelRepository, MarkTypeRepository, PredictionMarkRepository, PredictionRepository,
};
use spl_domain::ports::repositories::image::ImageRepository;
use spl_domain::ports::repositories::user::UserRepository;
use spl_shared::error::{AppError, Result};
use spl_shared::traits::IntoWithContext;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

pub struct PredictionService {
    prediction_repo: Arc<dyn PredictionRepository>,
    user_repo: Arc<dyn UserRepository>,
    image_repo: Arc<dyn ImageRepository>,
    label_repo: Arc<dyn LabelRepository>,
    mark_repo: Arc<dyn PredictionMarkRepository>,
    mark_type_repo: Arc<dyn MarkTypeRepository>,
    storage_client: Arc<dyn BlobStorageClient>,
    model_client: Arc<dyn ModelPredictionClient>,
    access_control: Arc<AccessControlService>,
}

impl PredictionService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        prediction_repo: Arc<dyn PredictionRepository>,
        user_repo: Arc<dyn UserRepository>,
        image_repo: Arc<dyn ImageRepository>,
        label_repo: Arc<dyn LabelRepository>,
        mark_repo: Arc<dyn PredictionMarkRepository>,
        mark_type_repo: Arc<dyn MarkTypeRepository>,
        storage_client: Arc<dyn BlobStorageClient>,
        model_client: Arc<dyn ModelPredictionClient>,
        access_control: Arc<AccessControlService>,
    ) -> Self {
        Self {
            prediction_repo,
            user_repo,
            image_repo,
            label_repo,
            mark_repo,
            mark_type_repo,
            storage_client,
            model_client,
            access_control,
        }
    }

    pub async fn create(&self, dto: CreatePredictionDto) -> Result<Prediction> {
        // Resolve entities from IDs concurrently

        let (user_opt, image_opt, label_opt) = tokio::try_join!(
            self.user_repo.get_by_id(dto.user_id),
            self.image_repo.get_by_id(dto.image_id),
            self.label_repo.get_by_id(dto.label_id),
        )?;

        let user = user_opt
            .ok_or_else(|| AppError::NotFound(format!("User {} not found", dto.user_id)))?;
        let image = image_opt
            .ok_or_else(|| AppError::NotFound(format!("Image {} not found", dto.image_id)))?;
        let label = label_opt
            .ok_or_else(|| AppError::NotFound(format!("Label {} not found", dto.label_id)))?;

        let context = CreatePredictionContext { user, image, label };
        let prediction = dto.into_with_context(context)?;

        self.prediction_repo.create(prediction).await
    }

    pub async fn predict_and_create(
        &self,
        user_id: Uuid,
        image_bytes: Vec<u8>,
        filename: String,
    ) -> Result<Prediction> {
        // 1. Validate user
        let user = self
            .user_repo
            .get_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;

        // 2. ML Prediction
        let result = self.model_client.predict(&image_bytes).await?;

        // 3. Helper to determine file paths
        let now = chrono::Utc::now();
        let filesdir = format!("{}/images/{}", user.id, now.format("%Y-%m-%d_%H-%M-%S"));
        let image_path = format!("{}/image.jpg", filesdir);

        // 4. Upload original image
        // In legacy, we re-encode the image from the result to ensure it matches what the model saw/processed
        // (legacy: cv2.imencode(".jpg", result.image))
        // Here, result.image is Bytes.
        self.storage_client
            .upload(result.image.clone(), &image_path)
            .await?;

        // 5. Calculate Severity and Label
        // Legacy: max(0.0, min(100.0, result.severity))
        // Result severity is 0-100.
        let severity = result.severity.max(0.0).min(100.0);
        let label = self
            .label_repo
            .get_by_severity(severity)
            .await?
            .ok_or_else(|| AppError::Unknown("No label found for severity".into()))?;

        // Upload masks and create Mark entities
        // Legacy names: "leaf", "lt_blg_lesion"
        // Rust result: leaf_mask, lesion_mask
        let leaf_mask_path = format!("{}/leaf_mask.jpg", filesdir);
        let lesion_mask_path = format!("{}/lt_blg_lesion_mask.jpg", filesdir);

        tokio::try_join!(
            self.storage_client
                .upload(result.leaf_mask, &leaf_mask_path),
            self.storage_client
                .upload(result.lesion_mask, &lesion_mask_path),
        )?;

        let (leaf_type_opt, lesion_type_opt) = tokio::try_join!(
            self.mark_type_repo.get_by_name("leaf_mask"),
            self.mark_type_repo.get_by_name("lt_blg_lesion_mask"),
        )?;

        let leaf_type = leaf_type_opt
            .ok_or_else(|| AppError::Unknown("Mark type leaf_mask not found".to_string()))?;

        let lesion_type = lesion_type_opt.ok_or_else(|| {
            AppError::Unknown("Mark type lt_blg_lesion_mask not found".to_string())
        })?;

        let marks = vec![
            (leaf_type, "leaf", leaf_mask_path),
            (lesion_type, "lt_blg_lesion", lesion_mask_path),
        ]
        .iter()
        .map(|(tp, name, path)| PredictionMark {
            id: Uuid::new_v4(),
            data: serde_json::json!({
                "filepath": path,
                "filename": format!("{}_mask.jpg", name),
            }),
            mark_type: tp.clone(),
            prediction_id: Uuid::nil(), // Will set later
            created_at: chrono::Utc::now(),
        })
        .collect::<Vec<_>>();

        // 7. Save Image Entity
        let image = Image {
            id: Uuid::new_v4(),
            user_id,
            filename,
            filepath: image_path,
            created_at: chrono::Utc::now(),
            prediction_id: None, // Set later
        };
        let image = self.image_repo.create(image).await?;

        // 8. Save Prediction Entity
        let mut prediction = Prediction {
            id: Uuid::new_v4(),
            user,
            image: image.clone(),
            label,
            plot_id: None,
            // Legacy: lesion_confidence.presence
            presence_confidence: result.lesion_confidence,
            // Legacy: lesion_confidence.absence
            absence_confidence: 1.0 - result.lesion_confidence,
            severity,
            feedback: None,
            created_at: chrono::Utc::now(),
            marks: vec![],
        };

        prediction = self.prediction_repo.create(prediction).await?;

        // 9. Update Relationships
        let mut image = image;
        image.prediction_id = Some(prediction.id);

        // 10. Update Marks with prediction_id and Save
        let marks: Vec<PredictionMark> = marks
            .into_iter()
            .map(|mut m| {
                m.prediction_id = prediction.id;
                m
            })
            .collect();

        tokio::try_join!(
            self.image_repo.update(image.clone()),
            self.mark_repo.create_many(marks.clone())
        )?;

        prediction.image = image; // Update prediction with image that now has prediction_id
        prediction.marks = marks; // Add marks to prediction
        Ok(prediction)
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<Prediction>> {
        self.prediction_repo.get_by_id(id).await
    }

    pub async fn get_by_user_id(&self, user_id: Uuid) -> Result<Vec<Prediction>> {
        self.prediction_repo.get_by_user_id(user_id).await
    }

    pub async fn get_by_user_id_and_id(
        &self,
        user_id: Uuid,
        id: Uuid,
    ) -> Result<Option<Prediction>> {
        self.prediction_repo
            .get_by_user_id_and_id(user_id, id)
            .await
    }

    pub async fn filter(
        &self,
        dto: FilterPredictionDto,
        requester: &User,
    ) -> Result<(u64, Vec<Prediction>)> {
        // Determine target users using AccessControlService
        let mut target_user_ids = self
            .access_control
            .get_accessible_user_ids(requester, dto.company_id)
            .await?;

        // If specific target users requested
        if let Some(requested_users) = dto.target_user_ids {
            if requester.role.level >= 50 {
                // Supervisor/Admin can filter within their scope
                let allowed_set: std::collections::HashSet<_> =
                    target_user_ids.into_iter().collect();

                target_user_ids = requested_users
                    .into_iter()
                    .filter(|uid| allowed_set.contains(uid))
                    .collect();
            }
        }

        if target_user_ids.is_empty() {
            return Ok((0, Vec::new()));
        }

        let limit = dto.limit.unwrap_or(16);
        let page = dto.page.unwrap_or(1);
        let offset = (page - 1) * limit;

        self.prediction_repo
            .filter(
                target_user_ids,
                dto.labels,
                dto.plot_ids,
                dto.min_date,
                dto.max_date,
                offset,
                limit,
            )
            .await
    }

    pub async fn get_all(&self, requester: &User) -> Result<Vec<Prediction>> {
        // Only admins can get all predictions
        if requester.role.level < 100 {
            return Err(AppError::Forbidden);
        }
        self.prediction_repo.get_all().await
    }

    pub async fn delete(&self, user_id: Uuid, id: Uuid) -> Result<Prediction> {
        let prediction = self
            .prediction_repo
            .get_by_user_id_and_id(user_id, id)
            .await?
            .ok_or_else(|| AppError::NotFound("Prediction not found".to_string()))?;

        // Delete from DB first
        let deleted = self.prediction_repo.delete(prediction.id).await?;

        // Rust path: deleted.image.filepath

        if let Some(parent) = std::path::Path::new(&deleted.image.filepath).parent() {
            let dir_path = parent.to_string_lossy().to_string().replace("\\", "/");
            if let Err(e) = self.storage_client.delete_directory(&dir_path).await {
                error!("Failed to delete blob directory {}: {}", dir_path, e);
            }
        }

        Ok(deleted)
    }
}
