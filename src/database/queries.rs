use sqlx::{PgPool, Row};
use chrono::{Utc, DateTime};

use super::models::{DbUser, CreateUser, DbTemplate, CreateTemplate, DbSubmission, CreateSubmission, DbSubmitter, CreateSubmitter, DbSignaturePosition, DbSignatureData};
use crate::models::signature::{CreateSignaturePosition, CreateSignatureData};

pub struct UserQueries;

impl UserQueries {
    pub async fn create_user(pool: &PgPool, user_data: CreateUser) -> Result<DbUser, sqlx::Error> {
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO users (name, email, password_hash, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name, email, password_hash, created_at, updated_at
            "#
        )
        .bind(&user_data.name)
        .bind(&user_data.email)
        .bind(&user_data.password_hash)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await?;

        Ok(DbUser {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            email: row.try_get("email")?,
            password_hash: row.try_get("password_hash")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }

    pub async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<Option<DbUser>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, name, email, password_hash, created_at, updated_at FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => Ok(Some(DbUser {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                email: row.try_get("email")?,
                password_hash: row.try_get("password_hash")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            })),
            None => Ok(None),
        }
    }
}

pub struct TemplateQueries;

impl TemplateQueries {
    pub async fn create_template(pool: &PgPool, template_data: CreateTemplate) -> Result<DbTemplate, sqlx::Error> {
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO templates (name, slug, user_id, fields)
            VALUES ($1, $2, $3, $4)
            RETURNING id, name, slug, user_id, fields, created_at, updated_at
            "#
        )
        .bind(&template_data.name)
        .bind(&template_data.slug)
        .bind(&template_data.user_id)
        .bind(&template_data.fields)
        .fetch_one(pool)
        .await?;

        Ok(DbTemplate {
            id: row.get(0),
            name: row.get(1),
            slug: row.get(2),
            user_id: row.get(3),
            fields: row.get(4),
            submitters: None, // No longer stored in templates
            documents: None, // Not inserted
            created_at: row.get(5),
            updated_at: row.get(6),
        })
    }

    pub async fn get_template_by_id(pool: &PgPool, id: i64) -> Result<Option<DbTemplate>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, name, slug, user_id, fields, documents, created_at, updated_at FROM templates WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => Ok(Some(DbTemplate {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                slug: row.try_get("slug")?,
                user_id: row.try_get("user_id")?,
                fields: row.try_get("fields")?,
                submitters: None, // No longer stored in templates
                documents: row.try_get("documents")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            })),
            None => Ok(None),
        }
    }

    pub async fn get_template_by_slug(pool: &PgPool, slug: &str) -> Result<Option<DbTemplate>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, name, slug, user_id, fields, documents, created_at, updated_at FROM templates WHERE slug = $1"
        )
        .bind(slug)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => Ok(Some(DbTemplate {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                slug: row.try_get("slug")?,
                user_id: row.try_get("user_id")?,
                fields: row.try_get("fields")?,
                submitters: None, // No longer stored in templates
                documents: row.try_get("documents")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            })),
            None => Ok(None),
        }
    }

    pub async fn get_all_templates(pool: &PgPool, user_id: i64) -> Result<Vec<DbTemplate>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, name, slug, user_id, fields, documents, created_at, updated_at FROM templates WHERE user_id = $1 ORDER BY created_at DESC "
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let mut templates = Vec::new();
        for row in rows {
            templates.push(DbTemplate {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                slug: row.try_get("slug")?,
                user_id: row.try_get("user_id")?,
                fields: row.try_get("fields")?,
                submitters: None, // No longer stored in templates
                documents: row.try_get("documents")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            });
        }
        Ok(templates)
    }

    pub async fn update_template(pool: &PgPool, id: i64, user_id: i64, name: Option<&str>, fields: Option<&serde_json::Value>) -> Result<Option<DbTemplate>, sqlx::Error> {
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            UPDATE templates
            SET name = COALESCE($3, name),
                fields = COALESCE($4, fields),
                updated_at = $5
            WHERE id = $1 AND user_id = $2
            RETURNING id, name, slug, user_id, fields, documents, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(user_id)
        .bind(name)
        .bind(fields)
        .bind(now)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => Ok(Some(DbTemplate {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                slug: row.try_get("slug")?,
                user_id: row.try_get("user_id")?,
                fields: row.try_get("fields")?,
                submitters: None, // No longer stored in templates
                documents: row.try_get("documents")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            })),
            None => Ok(None),
        }
    }

    pub async fn delete_template(pool: &PgPool, id: i64, user_id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM templates WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn clone_template(pool: &PgPool, original_id: i64, user_id: i64, new_name: &str, new_slug: &str) -> Result<Option<DbTemplate>, sqlx::Error> {
        // First get the original template
        if let Some(original) = Self::get_template_by_id(pool, original_id).await? {
            // Check if the template belongs to the user
            if original.user_id != user_id {
                return Ok(None);
            }

            let now = Utc::now();
            let create_data = CreateTemplate {
                name: new_name.to_string(),
                slug: new_slug.to_string(),
                user_id: user_id,
                fields: original.fields,
                documents: original.documents,
            };

            Self::create_template(pool, create_data).await.map(Some)
        } else {
            Ok(None)
        }
    }
}

pub struct SubmissionQueries;

impl SubmissionQueries {
    pub async fn create_submission(pool: &PgPool, submission_data: CreateSubmission) -> Result<DbSubmission, sqlx::Error> {
        let now = Utc::now();
        let row = sqlx::query(
            "INSERT INTO submissions (template_id, user_id, status, documents, created_at, updated_at, expires_at) 
             VALUES ($1, $2, $3, $4, $5, $6, $7) 
             RETURNING id, template_id, user_id, status, documents, created_at, updated_at, expires_at "
        )
        .bind(submission_data.template_id)
        .bind(submission_data.user_id)
        .bind(submission_data.status)
        .bind(submission_data.documents)
        .bind(now)
        .bind(now)
        .bind(submission_data.expires_at)
        .fetch_one(pool)
        .await?;

        Ok(DbSubmission {
            id: row.get(0),
            template_id: row.get(1),
            user_id: row.get(2),
            status: row.get(3),
            documents: row.get(4),
            created_at: row.get(5),
            updated_at: row.get(6),
            expires_at: row.get(7),
        })
    }

    pub async fn get_submissions_by_user(pool: &PgPool, user_id: i64) -> Result<Vec<DbSubmission>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, template_id, user_id, status, documents, created_at, updated_at, expires_at 
             FROM submissions WHERE user_id = $1 ORDER BY created_at DESC "
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let mut submissions = Vec::new();
        for row in rows {
            submissions.push(DbSubmission {
                id: row.get(0),
                template_id: row.get(1),
                user_id: row.get(2),
                status: row.get(3),
                documents: row.get(4),
                created_at: row.get(5),
                updated_at: row.get(6),
                expires_at: row.get(7),
            });
        }
        Ok(submissions)
    }

    pub async fn get_submissions_by_template(pool: &PgPool, template_id: i64) -> Result<Vec<DbSubmission>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, template_id, user_id, status, documents, created_at, updated_at, expires_at 
             FROM submissions WHERE template_id = $1 ORDER BY created_at DESC "
        )
        .bind(template_id)
        .fetch_all(pool)
        .await?;

        let mut submissions = Vec::new();
        for row in rows {
            submissions.push(DbSubmission {
                id: row.get(0),
                template_id: row.get(1),
                user_id: row.get(2),
                status: row.get(3),
                documents: row.get(4),
                created_at: row.get(5),
                updated_at: row.get(6),
                expires_at: row.get(7),
            });
        }
        Ok(submissions)
    }

    pub async fn get_submission(pool: &PgPool, id: i64, user_id: i64) -> Result<Option<DbSubmission>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, template_id, user_id, status, documents, created_at, updated_at, expires_at 
             FROM submissions WHERE id = $1 AND user_id = $2"
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(DbSubmission {
                id: row.get(0),
                template_id: row.get(1),
                user_id: row.get(2),
                status: row.get(3),
                documents: row.get(4),
                created_at: row.get(5),
                updated_at: row.get(6),
                expires_at: row.get(7),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_submission(pool: &PgPool, id: i64, user_id: i64, status: Option<&str>) -> Result<Option<DbSubmission>, sqlx::Error> {
        let now = Utc::now();
        let row = sqlx::query(
            "UPDATE submissions SET status = COALESCE($1, status), updated_at = $2 
             WHERE id = $3 AND user_id = $4 
             RETURNING id, template_id, user_id, status, documents, created_at, updated_at, expires_at "
        )
        .bind(status)
        .bind(now)
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(DbSubmission {
                id: row.get(0),
                template_id: row.get(1),
                user_id: row.get(2),
                status: row.get(3),
                documents: row.get(4),
                created_at: row.get(5),
                updated_at: row.get(6),
                expires_at: row.get(7),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_submission(pool: &PgPool, id: i64, user_id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM submissions WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_submission_by_id(pool: &PgPool, id: i64) -> Result<Option<DbSubmission>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, template_id, user_id, status, documents, created_at, updated_at, expires_at 
             FROM submissions WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(DbSubmission {
                id: row.get(0),
                template_id: row.get(1),
                user_id: row.get(2),
                status: row.get(3),
                documents: row.get(4),
                created_at: row.get(5),
                updated_at: row.get(6),
                expires_at: row.get(7),
            }))
        } else {
            Ok(None)
        }
    }
}

pub struct SubmitterQueries;

impl SubmitterQueries {
    pub async fn create_submitter(pool: &PgPool, submitter_data: CreateSubmitter) -> Result<DbSubmitter, sqlx::Error> {
        let now = Utc::now();
        let row = sqlx::query(
            "INSERT INTO submitters (submission_id, name, email, status, signed_at, token, fields_data, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) 
             RETURNING id, submission_id, name, email, status, signed_at, token, fields_data, created_at, updated_at "
        )
        .bind(submitter_data.submission_id)
        .bind(submitter_data.name)
        .bind(submitter_data.email)
        .bind(submitter_data.status)
        .bind(None::<DateTime<Utc>>)
        .bind(submitter_data.token)
        .bind(submitter_data.fields_data)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await?;

        Ok(DbSubmitter {
            id: row.get(0),
            submission_id: row.get(1),
            name: row.get(2),
            email: row.get(3),
            status: row.get(4),
            signed_at: row.get(5),
            token: row.get(6),
            fields_data: row.get(7),
            created_at: row.get(8),
            updated_at: row.get(9),
        })
    }

    pub async fn get_submitters_by_submission(pool: &PgPool, submission_id: i64) -> Result<Vec<DbSubmitter>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, submission_id, name, email, status, signed_at, token, fields_data, created_at, updated_at 
             FROM submitters WHERE submission_id = $1 ORDER BY created_at "
        )
        .bind(submission_id)
        .fetch_all(pool)
        .await?;

        let mut submitters = Vec::new();
        for row in rows {
            submitters.push(DbSubmitter {
                id: row.get(0),
                submission_id: row.get(1),
                name: row.get(2),
                email: row.get(3),
                status: row.get(4),
                signed_at: row.get(5),
                token: row.get(6),
                fields_data: row.get(7),
                created_at: row.get(8),
                updated_at: row.get(9),
            });
        }
        Ok(submitters)
    }

    pub async fn get_submitter_by_token(pool: &PgPool, token: &str) -> Result<Option<DbSubmitter>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, submission_id, name, email, status, signed_at, token, fields_data, created_at, updated_at 
             FROM submitters WHERE token = $1"
        )
        .bind(token)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(DbSubmitter {
                id: row.get(0),
                submission_id: row.get(1),
                name: row.get(2),
                email: row.get(3),
                status: row.get(4),
                signed_at: row.get(5),
                token: row.get(6),
                fields_data: row.get(7),
                created_at: row.get(8),
                updated_at: row.get(9),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_submitter(pool: &PgPool, id: i64, status: Option<&str>, fields_data: Option<&serde_json::Value>) -> Result<Option<DbSubmitter>, sqlx::Error> {
        let now = Utc::now();
        let signed_at = if status == Some("signed") { Some(now) } else { None };
        
        let row = sqlx::query(
            "UPDATE submitters SET status = COALESCE($1, status), fields_data = COALESCE($2, fields_data), signed_at = COALESCE($3, signed_at), updated_at = $4 
             WHERE id = $5 
             RETURNING id, submission_id, name, email, status, signed_at, token, fields_data, created_at, updated_at "
        )
        .bind(status)
        .bind(fields_data)
        .bind(signed_at)
        .bind(now)
        .bind(id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(DbSubmitter {
                id: row.get(0),
                submission_id: row.get(1),
                name: row.get(2),
                email: row.get(3),
                status: row.get(4),
                signed_at: row.get(5),
                token: row.get(6),
                fields_data: row.get(7),
                created_at: row.get(8),
                updated_at: row.get(9),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_submitter_by_id(pool: &PgPool, id: i64) -> Result<Option<DbSubmitter>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, submission_id, name, email, status, signed_at, token, fields_data, created_at, updated_at 
             FROM submitters WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(DbSubmitter {
                id: row.get(0),
                submission_id: row.get(1),
                name: row.get(2),
                email: row.get(3),
                status: row.get(4),
                signed_at: row.get(5),
                token: row.get(6),
                fields_data: row.get(7),
                created_at: row.get(8),
                updated_at: row.get(9),
            }))
        } else {
            Ok(None)
        }
    }
}

pub struct SignatureQueries;

impl SignatureQueries {
    pub async fn create_signature_position(
        pool: &PgPool,
        position_data: CreateSignaturePosition,
    ) -> Result<DbSignaturePosition, sqlx::Error> {
        let now = Utc::now();

        // Get the next version for this field
        let next_version = sqlx::query_scalar::<_, i32>(
            r#"
            SELECT COALESCE(MAX(version), 0) + 1
            FROM signature_positions
            WHERE submitter_id = $1 AND field_name = $2
            "#
        )
        .bind(position_data.submitter_id)
        .bind(&position_data.field_name)
        .fetch_one(pool)
        .await?;

        // Deactivate previous signatures for this field
        sqlx::query(
            r#"
            UPDATE signature_positions
            SET is_active = false
            WHERE submitter_id = $1 AND field_name = $2 AND is_active = true
            "#
        )
        .bind(position_data.submitter_id)
        .bind(&position_data.field_name)
        .execute(pool)
        .await?;

        let row = sqlx::query(
            r#"
            INSERT INTO signature_positions (submitter_id, field_name, page, x, y, width, height, signature_value, signed_at, ip_address, user_agent, version, is_active, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING id, submitter_id, field_name, page, x, y, width, height, signature_value, signed_at, ip_address, user_agent, version, is_active, created_at
            "#
        )
        .bind(position_data.submitter_id)
        .bind(&position_data.field_name)
        .bind(position_data.page)
        .bind(position_data.x)
        .bind(position_data.y)
        .bind(position_data.width)
        .bind(position_data.height)
        .bind(&position_data.signature_value)
        .bind(now)
        .bind(&position_data.ip_address)
        .bind(&position_data.user_agent)
        .bind(next_version)
        .bind(true)
        .bind(now)
        .fetch_one(pool)
        .await?;

        Ok(DbSignaturePosition {
            id: row.try_get("id")?,
            submitter_id: row.try_get("submitter_id")?,
            field_name: row.try_get("field_name")?,
            page: row.try_get("page")?,
            x: row.try_get("x")?,
            y: row.try_get("y")?,
            width: row.try_get("width")?,
            height: row.try_get("height")?,
            signature_value: row.try_get("signature_value")?,
            signed_at: row.try_get("signed_at")?,
            ip_address: row.try_get("ip_address")?,
            user_agent: row.try_get("user_agent")?,
            version: row.try_get("version")?,
            is_active: row.try_get("is_active")?,
            created_at: row.try_get("created_at")?,
        })
    }

    pub async fn create_signature_data(
        pool: &PgPool,
        signature_data: CreateSignatureData,
    ) -> Result<DbSignatureData, sqlx::Error> {
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO signature_data (submitter_id, signature_image, signature_value, signed_at, ip_address, user_agent)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, submitter_id, signature_value, signed_at, ip_address, user_agent
            "#
        )
        .bind(signature_data.submitter_id)
        .bind(&signature_data.signature_value)
        .bind(now)
        .bind(&signature_data.ip_address)
        .bind(&signature_data.user_agent)
        .fetch_one(pool)
        .await?;

        Ok(DbSignatureData {
            id: row.try_get("id")?,
            submitter_id: row.try_get("submitter_id")?,
            signature_value: row.try_get("signature_value")?,
            signed_at: row.try_get("signed_at")?,
            ip_address: row.try_get("ip_address")?,
            user_agent: row.try_get("user_agent")?,
        })
    }

    pub async fn get_signature_positions_by_submitter(
        pool: &PgPool,
        submitter_id: i64,
    ) -> Result<Vec<DbSignaturePosition>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, submitter_id, field_name, page, x, y, width, height, signature_value, signed_at, ip_address, user_agent, version, is_active, created_at
            FROM signature_positions
            WHERE submitter_id = $1
            ORDER BY field_name, version DESC
            "#
        )
        .bind(submitter_id)
        .fetch_all(pool)
        .await?;

        let mut positions = Vec::new();
        for row in rows {
            positions.push(DbSignaturePosition {
                id: row.try_get("id")?,
                submitter_id: row.try_get("submitter_id")?,
                field_name: row.try_get("field_name")?,
                page: row.try_get("page")?,
                x: row.try_get("x")?,
                y: row.try_get("y")?,
                width: row.try_get("width")?,
                height: row.try_get("height")?,
                signature_value: row.try_get("signature_value")?,
                signed_at: row.try_get("signed_at")?,
                ip_address: row.try_get("ip_address")?,
                user_agent: row.try_get("user_agent")?,
                version: row.try_get("version")?,
                is_active: row.try_get("is_active")?,
                created_at: row.try_get("created_at")?,
            });
        }

        Ok(positions)
    }

    pub async fn get_signature_history_by_field(
        pool: &PgPool,
        submitter_id: i64,
        field_name: &str,
    ) -> Result<Vec<DbSignaturePosition>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, submitter_id, field_name, page, x, y, width, height, signature_value, signed_at, ip_address, user_agent, version, is_active, created_at
            FROM signature_positions
            WHERE submitter_id = $1 AND field_name = $2
            ORDER BY version DESC
            "#
        )
        .bind(submitter_id)
        .bind(field_name)
        .fetch_all(pool)
        .await?;

        let mut positions = Vec::new();
        for row in rows {
            positions.push(DbSignaturePosition {
                id: row.try_get("id")?,
                submitter_id: row.try_get("submitter_id")?,
                field_name: row.try_get("field_name")?,
                page: row.try_get("page")?,
                x: row.try_get("x")?,
                y: row.try_get("y")?,
                width: row.try_get("width")?,
                height: row.try_get("height")?,
                signature_value: row.try_get("signature_value")?,
                signed_at: row.try_get("signed_at")?,
                ip_address: row.try_get("ip_address")?,
                user_agent: row.try_get("user_agent")?,
                version: row.try_get("version")?,
                is_active: row.try_get("is_active")?,
                created_at: row.try_get("created_at")?,
            });
        }

        Ok(positions)
    }

    pub async fn get_latest_signature_by_field(
        pool: &PgPool,
        submitter_id: i64,
        field_name: &str,
    ) -> Result<Option<DbSignaturePosition>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, submitter_id, field_name, page, x, y, width, height, signature_value, signed_at, ip_address, user_agent, version, is_active, created_at
            FROM signature_positions
            WHERE submitter_id = $1 AND field_name = $2 AND is_active = true
            ORDER BY version DESC
            LIMIT 1
            "#
        )
        .bind(submitter_id)
        .bind(field_name)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(DbSignaturePosition {
                id: row.try_get("id")?,
                submitter_id: row.try_get("submitter_id")?,
                field_name: row.try_get("field_name")?,
                page: row.try_get("page")?,
                x: row.try_get("x")?,
                y: row.try_get("y")?,
                width: row.try_get("width")?,
                height: row.try_get("height")?,
                signature_value: row.try_get("signature_value")?,
                signed_at: row.try_get("signed_at")?,
                ip_address: row.try_get("ip_address")?,
                user_agent: row.try_get("user_agent")?,
                version: row.try_get("version")?,
                is_active: row.try_get("is_active")?,
                created_at: row.try_get("created_at")?,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_signature_data_by_submitter(
        pool: &PgPool,
        submitter_id: i64,
    ) -> Result<Option<DbSignatureData>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, submitter_id, signature_image, signature_value, signed_at, ip_address, user_agent
            FROM signature_data
            WHERE submitter_id = $1
            ORDER BY signed_at DESC
            LIMIT 1
            "#
        )
        .bind(submitter_id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(DbSignatureData {
                id: row.try_get("id")?,
                submitter_id: row.try_get("submitter_id")?,
                signature_value: row.try_get("signature_value")?,
                signed_at: row.try_get("signed_at")?,
                ip_address: row.try_get("ip_address")?,
                user_agent: row.try_get("user_agent")?,
            }))
        } else {
            Ok(None)
        }
    }
}
